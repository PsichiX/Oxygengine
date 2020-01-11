use js_sys::{Array, Object, Reflect};
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;

pub use serde_json::{
    json, map::Map as ScriptableMap, value::Value as ScriptableValue, Number as ScriptableNumber,
};

#[derive(Debug, Clone)]
pub enum ScriptableError {
    CouldNotSerialize,
    CouldNotDeserialize,
}

pub trait Scriptable {
    fn from_scriptable(_data: &ScriptableValue) -> Result<Self, ScriptableError>
    where
        Self: Sized,
    {
        Err(ScriptableError::CouldNotDeserialize)
    }

    fn to_scriptable(&self) -> Result<ScriptableValue, ScriptableError> {
        Err(ScriptableError::CouldNotSerialize)
    }

    fn to_js(&self) -> Result<JsValue, ScriptableError> {
        if let Ok(value) = self.to_scriptable() {
            scriptable_value_to_js(&value)
        } else {
            Err(ScriptableError::CouldNotSerialize)
        }
    }

    fn from_js(js: JsValue) -> Result<Self, ScriptableError>
    where
        Self: Sized,
    {
        let value = scriptable_js_to_value(js)?;
        Self::from_scriptable(&value)
    }
}

impl<T> Scriptable for T
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn from_scriptable(data: &ScriptableValue) -> Result<Self, ScriptableError> {
        if let Ok(r) = serde_json::from_value::<T>(data.clone()) {
            Ok(r)
        } else {
            Err(ScriptableError::CouldNotDeserialize)
        }
    }

    fn to_scriptable(&self) -> Result<ScriptableValue, ScriptableError> {
        if let Ok(v) = serde_json::to_value::<T>(self.clone()) {
            Ok(v)
        } else {
            Err(ScriptableError::CouldNotSerialize)
        }
    }
}

pub fn scriptable_value_to_js(value: &ScriptableValue) -> Result<JsValue, ScriptableError> {
    match value {
        ScriptableValue::Null => Ok(JsValue::NULL),
        ScriptableValue::Bool(v) => Ok(JsValue::from_bool(*v)),
        ScriptableValue::Number(v) => {
            if let Some(v) = v.as_f64() {
                Ok(JsValue::from_f64(v))
            } else {
                Err(ScriptableError::CouldNotSerialize)
            }
        }
        ScriptableValue::String(v) => Ok(JsValue::from_str(v)),
        ScriptableValue::Array(v) => scriptable_array_to_js(v),
        ScriptableValue::Object(v) => scriptable_map_to_js(v),
    }
}

pub fn scriptable_js_to_value(js: JsValue) -> Result<ScriptableValue, ScriptableError> {
    if js.is_null() || js.is_undefined() {
        Ok(ScriptableValue::Null)
    } else if let Some(v) = js.as_bool() {
        Ok(ScriptableValue::Bool(v))
    } else if let Some(v) = js.as_f64() {
        if let Some(v) = ScriptableNumber::from_f64(v) {
            Ok(ScriptableValue::Number(v))
        } else {
            Err(ScriptableError::CouldNotDeserialize)
        }
    } else if let Some(v) = js.as_string() {
        Ok(ScriptableValue::String(v))
    } else if Array::is_array(&js) {
        scriptable_js_to_array(js)
    } else if js.is_object() {
        scriptable_js_to_map(js)
    } else {
        Err(ScriptableError::CouldNotDeserialize)
    }
}

fn scriptable_array_to_js(value: &[ScriptableValue]) -> Result<JsValue, ScriptableError> {
    let result = Array::new_with_length(value.len() as u32);
    for (i, item) in value.iter().enumerate() {
        result.set(i as u32, scriptable_value_to_js(item)?);
    }
    Ok(result.into())
}

fn scriptable_js_to_array(js: JsValue) -> Result<ScriptableValue, ScriptableError> {
    let items = Array::from(&js)
        .iter()
        .map(|item| scriptable_js_to_value(item))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ScriptableValue::Array(items))
}

fn scriptable_map_to_js(
    value: &ScriptableMap<String, ScriptableValue>,
) -> Result<JsValue, ScriptableError> {
    let result: JsValue = Object::new().into();
    for (key, value) in value {
        if Reflect::set(
            &result,
            &JsValue::from_str(key),
            &scriptable_value_to_js(value)?,
        )
        .is_err()
        {
            return Err(ScriptableError::CouldNotSerialize);
        }
    }
    Ok(result)
}

fn scriptable_js_to_map(js: JsValue) -> Result<ScriptableValue, ScriptableError> {
    if let Ok(keys) = Reflect::own_keys(&js) {
        let items = keys
            .iter()
            .map(|key| {
                if let Ok(v) = Reflect::get(&js, &key) {
                    Ok((key.as_string().unwrap(), scriptable_js_to_value(v)?))
                } else {
                    Err(ScriptableError::CouldNotDeserialize)
                }
            })
            .collect::<Result<ScriptableMap<_, _>, _>>()?;
        Ok(ScriptableValue::Object(items))
    } else {
        Err(ScriptableError::CouldNotDeserialize)
    }
}
