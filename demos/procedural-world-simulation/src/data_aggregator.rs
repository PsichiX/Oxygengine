use std::{fs::File, io::Write, path::Path};

pub trait DataAggregatorRenderer {
    fn render_columns(&self, columns: &mut Vec<String>);
}

impl DataAggregatorRenderer for f64 {
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.to_string());
    }
}

impl<T> DataAggregatorRenderer for Vec<T>
where
    T: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        for item in self {
            columns.push(item.to_string());
        }
    }
}

impl<T> DataAggregatorRenderer for [T]
where
    T: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        for item in self {
            columns.push(item.to_string());
        }
    }
}

impl<T0, T1> DataAggregatorRenderer for (T0, T1)
where
    T0: ToString,
    T1: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
    }
}

impl<T0, T1, T2> DataAggregatorRenderer for (T0, T1, T2)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
    }
}

impl<T0, T1, T2, T3> DataAggregatorRenderer for (T0, T1, T2, T3)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
    T3: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
        columns.push(self.3.to_string());
    }
}

impl<T0, T1, T2, T3, T4> DataAggregatorRenderer for (T0, T1, T2, T3, T4)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
    T3: ToString,
    T4: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
        columns.push(self.3.to_string());
        columns.push(self.4.to_string());
    }
}

impl<T0, T1, T2, T3, T4, T5> DataAggregatorRenderer for (T0, T1, T2, T3, T4, T5)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
    T3: ToString,
    T4: ToString,
    T5: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
        columns.push(self.3.to_string());
        columns.push(self.4.to_string());
        columns.push(self.5.to_string());
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> DataAggregatorRenderer for (T0, T1, T2, T3, T4, T5, T6)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
    T3: ToString,
    T4: ToString,
    T5: ToString,
    T6: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
        columns.push(self.3.to_string());
        columns.push(self.4.to_string());
        columns.push(self.5.to_string());
        columns.push(self.6.to_string());
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> DataAggregatorRenderer for (T0, T1, T2, T3, T4, T5, T6, T7)
where
    T0: ToString,
    T1: ToString,
    T2: ToString,
    T3: ToString,
    T4: ToString,
    T5: ToString,
    T6: ToString,
    T7: ToString,
{
    fn render_columns(&self, columns: &mut Vec<String>) {
        columns.push(self.0.to_string());
        columns.push(self.1.to_string());
        columns.push(self.2.to_string());
        columns.push(self.3.to_string());
        columns.push(self.4.to_string());
        columns.push(self.5.to_string());
        columns.push(self.6.to_string());
        columns.push(self.7.to_string());
    }
}

pub struct DataAggregator<T>
where
    T: DataAggregatorRenderer,
{
    result: File,
    rows: Vec<T>,
    auto_flush: Option<usize>,
}

impl<T> DataAggregator<T>
where
    T: DataAggregatorRenderer,
{
    pub fn new<P>(result_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            result: File::create(result_path).unwrap(),
            rows: vec![],
            auto_flush: None,
        }
    }

    pub fn auto_flush(&self) -> Option<usize> {
        self.auto_flush
    }

    pub fn set_auto_flush(&mut self, rows_count: Option<usize>) {
        self.auto_flush = rows_count;
    }

    pub fn push(&mut self, item: T) {
        self.rows.push(item);
        if let Some(count) = self.auto_flush {
            if self.rows.len() >= count {
                self.flush();
            }
        }
    }

    pub fn flush(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let mut columns = vec![];
        for row in &self.rows {
            columns.clear();
            row.render_columns(&mut columns);
            self.result
                .write_fmt(format_args!("{}\r\n", columns.join("\t")))
                .unwrap();
        }
        self.rows.clear();
        self.result.flush().unwrap();
    }
}

impl<T> Drop for DataAggregator<T>
where
    T: DataAggregatorRenderer,
{
    fn drop(&mut self) {
        self.flush();
    }
}
