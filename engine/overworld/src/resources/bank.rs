use crate::resources::market::Currency;
use oxygengine_core::id::ID;
use std::collections::HashMap;

pub type BankAccountId = ID<BankAccount<()>>;

#[derive(Debug, Clone)]
pub enum BankError<T>
where
    T: Currency,
{
    AccountDoesNotExists(BankAccountId),
    CouldNotDeposit(BankAccountId, T),
    CouldNotWithdraw(BankAccountId, T),
    CouldNotTransfer(BankAccountId, BankAccountId, T),
}

#[derive(Debug, Default, Clone)]
pub struct BankAccount<T>
where
    T: Currency + std::fmt::Debug + Default + Clone + Send + Sync,
{
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct Bank<T>
where
    T: Currency + std::fmt::Debug + Default + Clone + Send + Sync,
{
    accounts: HashMap<BankAccountId, BankAccount<T>>,
}

impl<T> Default for Bank<T>
where
    T: Currency + std::fmt::Debug + Default + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            accounts: Default::default(),
        }
    }
}

impl<T> Bank<T>
where
    T: Currency + std::fmt::Debug + Default + Clone + Send + Sync,
{
    pub fn create_account(&mut self) -> BankAccountId {
        let id = BankAccountId::new();
        self.accounts.insert(id, Default::default());
        id
    }

    pub fn remove_account(&mut self, id: BankAccountId) -> Result<T, BankError<T>> {
        match self.accounts.remove(&id) {
            Some(account) => Ok(account.value),
            None => Err(BankError::AccountDoesNotExists(id)),
        }
    }

    pub fn accounts(&self) -> impl Iterator<Item = (BankAccountId, &BankAccount<T>)> {
        self.accounts.iter().map(|(id, account)| (*id, account))
    }

    pub fn deposit(&mut self, id: BankAccountId, value: T) -> Result<(), BankError<T>> {
        match self.accounts.get_mut(&id) {
            Some(account) => match account.value.accumulate(value) {
                Ok(value) => {
                    account.value = value;
                    Ok(())
                }
                Err(_) => Err(BankError::CouldNotDeposit(id, value)),
            },
            None => Err(BankError::AccountDoesNotExists(id)),
        }
    }

    pub fn withdraw(&mut self, id: BankAccountId, value: T) -> Result<T, BankError<T>> {
        match self.accounts.get_mut(&id) {
            Some(account) => match account.value.take(value) {
                Ok((left, taken)) => {
                    account.value = left;
                    Ok(taken)
                }
                Err(_) => Err(BankError::CouldNotDeposit(id, value)),
            },
            None => Err(BankError::AccountDoesNotExists(id)),
        }
    }

    pub fn transfer(
        &mut self,
        from: BankAccountId,
        to: BankAccountId,
        value: T,
    ) -> Result<(), BankError<T>> {
        if from == to {
            return Ok(());
        }
        let from_value = match self.accounts.get(&from) {
            Some(from) => from.value,
            None => return Err(BankError::AccountDoesNotExists(from)),
        };
        let to_value = match self.accounts.get(&to) {
            Some(to) => to.value,
            None => return Err(BankError::AccountDoesNotExists(to)),
        };
        let (from_value, to_value) = match from_value.exchange(to_value, value) {
            Ok(result) => result,
            Err(_) => return Err(BankError::CouldNotTransfer(from, to, value)),
        };
        self.accounts.get_mut(&from).unwrap().value = from_value;
        self.accounts.get_mut(&to).unwrap().value = to_value;
        Ok(())
    }

    pub fn batched_transfers<'a>(
        &'a mut self,
        iter: impl Iterator<Item = (BankAccountId, BankAccountId, T)> + 'a,
    ) -> impl Iterator<Item = BankError<T>> + 'a {
        iter.filter_map(|(from, to, value)| match self.transfer(from, to, value) {
            Ok(_) => None,
            Err(error) => Some(error),
        })
    }
}
