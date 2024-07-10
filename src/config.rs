pub mod conf {
    use ini::Ini;
    use std::{collections::HashMap, path::Path};

    pub type Config = HashMap<Box<str>, Value>;

    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum Value {
        I32(i32),
        U32(u32),
        F32(f32),
        Bool(bool),
        String(Box<str>),
    }

    pub trait Unwrap<T> {
        fn unwrap(&self) -> T;
    }

    impl Unwrap<i32> for Value {
        fn unwrap(&self) -> i32 {
            if let Value::I32(val) = self {
                *val
            } else {
                panic!("Expected i32, got a different type")
            }
        }
    }

    impl Unwrap<u32> for Value {
        fn unwrap(&self) -> u32 {
            if let Value::U32(val) = self {
                *val
            } else {
                panic!("Expected u32, got a different type")
            }
        }
    }

    impl Unwrap<f32> for Value {
        fn unwrap(&self) -> f32 {
            if let Value::F32(val) = self {
                *val
            } else {
                panic!("Expected f32, got a different type")
            }
        }
    }

    impl Unwrap<bool> for Value {
        fn unwrap(&self) -> bool {
            if let Value::Bool(val) = self {
                *val
            } else {
                panic!("Expected bool, got a different type")
            }
        }
    }

    impl Unwrap<Box<str>> for Value {
        fn unwrap(&self) -> Box<str> {
            if let Value::String(val) = self {
                val.to_string().into()
            } else {
                panic!("Expected String, got a different type")
            }
        }
    }

    pub fn read(path: &Path, default_config: HashMap<&str, Value>) -> Config {
        let ini = Ini::load_from_file(path).unwrap_or_default();
        let default = ini::Properties::default();
        let user_config = ini.section::<String>(None).unwrap_or(&default);

        let mut config: Config = Config::default();
        for (k, v) in default_config {
            let value_str = match user_config.get(k) {
                Some(v) => v,
                None => {
                    config.insert(k.to_owned().into(), v);
                    continue;
                } // Fallback to default config
            };

            let value = match v {
                Value::I32(i) => Value::I32(value_str.parse::<i32>().unwrap_or(i)),
                Value::U32(i) => Value::U32(value_str.parse::<u32>().unwrap_or(i)),
                Value::F32(i) => Value::F32(value_str.parse::<f32>().unwrap_or(i)),
                Value::Bool(i) => Value::Bool(value_str.parse::<bool>().unwrap_or(i)),
                Value::String(_) => Value::String(value_str.to_owned().into()),
            };

            config.insert(k.to_string().into(), value);
        }

        config
    }
}
