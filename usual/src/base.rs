use anyhow::Error;

#[cfg(not(feature = "tokio-postgres"))]
pub trait TryGetRow {
    fn try_get<'a, T: 'static + Clone>(&'a self, index: &str) -> Result<T, Error>;
}

#[cfg(feature = "tokio-postgres")]
use tokio_postgres::{types::FromSql, Row};

#[cfg(feature = "tokio-postgres")]
pub trait TryGetRow {
    fn try_get<'a, T: 'static + Clone + FromSql<'a>>(&'a self, index: &str) -> Result<T, Error>;
}

#[cfg(feature = "tokio-postgres")]
impl TryGetRow for Row {
    fn try_get<'a, T: 'static + Clone + FromSql<'a>>(&'a self, index: &str) -> Result<T, Error> {
        self.try_get(index).map_err(|e| Error::from(e))
    }
}

pub trait IntoEnum<T> {
    fn as_enum(&self) -> T;
    fn as_enum_i32(&self) -> i32;
}

#[macro_export]
macro_rules! impl_model {
  ($struct:ident {
      $( pub $field:ident:$type:ty ),*
  }) => {
      #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
      #[serde(rename_all = "camelCase")]
      pub struct $struct {
          $(
              pub $field: $type,
          )*
      }

      impl Model for $struct {
          fn from_row_starting_index(_index: usize, row: &impl TryGetRow) -> Self {
            $struct {
                $(
                    $field: row.try_get(format!("{}{}", Self::prefix(), stringify!($field)).as_str())
                        .expect(&format!("You messed up while trying to get {} ({}{}) from {}", stringify!($field), Self::prefix(), stringify!($field), stringify!($struct)))
                ),+
              }
          }

          fn from_row_with_prefix(prefix: &str, row: &impl TryGetRow) -> Self {
            $struct {
                $(
                    $field: row.try_get(format!("{}{}", prefix, stringify!($field)).as_str())
                        .expect(&format!("You messed up while trying to get {} ({}{}) from {}", stringify!($field), prefix, stringify!($field), stringify!($struct)))
                ),+
            }
          }

          fn columns_list() -> Vec<&'static str> {
              vec![$(
                  stringify!($field)
              ),+]
          }

          fn prefix() -> &'static str {
            concat!(stringify!($struct), "__")
          }
      }
  };
}

pub trait Model
where
    Self: Sized,
{
    fn from_rows<T: TryGetRow>(rows: Vec<T>) -> Vec<Self> {
        rows.into_iter().map(|r| Self::from_row(&r)).collect()
    }

    fn from_row_starting_index(index: usize, row: &impl TryGetRow) -> Self;

    fn from_row_with_prefix(_prefix: &str, row: &impl TryGetRow) -> Self {
        Self::from_row_starting_index(0, row)
    }

    fn from_row(row: &impl TryGetRow) -> Self {
        Self::from_row_starting_index(0, row)
    }

    fn columns_list() -> Vec<&'static str>;

    fn prefix() -> &'static str {
        ""
    }

    fn columns_with_prefix_and_table(prefix: Option<&str>, table: Option<&str>) -> String {
        Self::columns_list()
            .iter()
            .map(|c| Self::column_with_prefix_and_table(c.to_owned(), prefix, table))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn column_with_prefix_and_table(
        column: &str,
        prefix: Option<&str>,
        table: Option<&str>,
    ) -> String {
        let mut column = match prefix {
            Some(val) => format!("{} as {}{}", column, val, column),
            None => column.to_owned(),
        };

        column = match table {
            Some(val) => format!("{}.{}", val, column),
            None => column.to_owned(),
        };

        column
    }

    fn columns() -> String {
        Self::columns_with_prefix_and_table(Some(Self::prefix()), None)
    }

    fn columns_with_prefix(prefix: &str) -> String {
        Self::columns_with_prefix_and_table(Some(prefix), None)
    }

    fn columns_with_table(table: &str) -> String {
        Self::columns_with_prefix_and_table(Some(Self::prefix()), Some(table))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Error};
    use std::{any::Any, collections::HashMap};
    use usual_proc::query;

    use super::{Model, TryGetRow};
    use crate::impl_model;

    struct Row {
        value: HashMap<String, Box<dyn Any>>,
    }

    impl TryGetRow for Row {
        fn try_get<T: 'static + Clone>(&self, index: &str) -> Result<T, Error> {
            let value = self.value.get(index).unwrap();
            let cast: T = (value as &Box<dyn Any>)
                .downcast_ref::<T>()
                .ok_or_else(|| {
                    anyhow!(format!(
                        "Attempted to get type {}, but was not the correct type",
                        std::any::type_name::<T>(),
                    ))
                })?
                .clone();

            Ok(cast)
        }
    }

    impl_model!(TestModel {
        pub some_string: String,
        pub some_int: i32
    });

    impl_model!(TestModel2 {
        pub key: String
    });

    #[test]
    fn it_should_be_able_to_get_from_row() {
        let some_string = "asdfasdfasdf".to_string();
        let some_int = 42;

        let mut value: HashMap<String, Box<dyn Any>> = HashMap::new();
        value.insert("some_string".to_owned(), Box::new(some_string.clone()));
        value.insert("some_int".to_owned(), Box::new(some_int));

        let row = Row { value };

        let test = TestModel::from_row_with_prefix("", &row);

        assert!(
            test.some_string == some_string,
            "It should correctly pull the string value from a row."
        );
        assert!(
            test.some_int == some_int,
            "It shoudl correctly pull the int value from a row"
        );
    }

    #[test]
    fn it_should_correctly_insert_columns() {
        let macro_output = query!("SELECT {TestModel} FROM test_model");

        assert!(macro_output == "SELECT some_string as TestModel__some_string, some_int as TestModel__some_int FROM test_model")
    }

    #[test]
    fn it_should_correctly_select_subsets_of_columns() {
        let macro_output = query!("SELECT {TestModel::some_string} FROM test_model");

        assert!(macro_output == "SELECT some_string as TestModel__some_string FROM test_model")
    }

    #[test]
    fn it_should_correctly_select_subsets_of_multiple_columns() {
        let macro_output = query!("SELECT {TestModel::some_string,some_int} FROM test_model");

        assert!(macro_output == "SELECT some_string as TestModel__some_string, some_int as TestModel__some_int FROM test_model")
    }

    #[test]
    fn it_should_correctly_insert_columns_with_a_table() {
        let macro_output = query!("SELECT {TestModel as t} FROM test_model as t");

        assert!(macro_output == "SELECT t.some_string as TestModel__some_string, t.some_int as TestModel__some_int FROM test_model as t")
    }

    #[test]
    fn it_should_correctly_insert_columns_with_multiple_tables() {
        let macro_output = query!("SELECT {TestModel as t}, {TestModel2 as t2} FROM test_model as t JOIN test_model as t2 on t.id = t2.id");

        println!("macro_output: {}", macro_output);

        assert!(macro_output == "SELECT t.some_string as TestModel__some_string, t.some_int as TestModel__some_int, t2.key as TestModel2__key FROM test_model as t JOIN test_model as t2 on t.id = t2.id")
    }

    #[test]
    fn it_should_correctly_select_subsets_of_columns_with_a_table() {
        let macro_output = query!("SELECT {TestModel::some_string as t} FROM test_model as t");

        assert!(
            macro_output == "SELECT t.some_string as TestModel__some_string FROM test_model as t"
        )
    }
}
