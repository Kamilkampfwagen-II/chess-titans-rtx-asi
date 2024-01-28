pub mod config {
    use std::collections::HashMap;
    use ini::Ini;


    #[derive(Debug)]
    pub enum Value {
        I32(i32),
        U32(u32),
        F32(f32),
        Bool(bool),
        String(String),
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

    impl Unwrap<String> for Value {
        fn unwrap(&self) -> String {
            if let Value::String(val) = self {
                val.to_string()
            } else {
                panic!("Expected String, got a different type")
            }
        }
    }


    pub fn read() -> HashMap<String, Value> {
        let default_config = HashMap::from([
            ("console"   , Value::Bool(true)),
            ("fov"       , Value::F32(90.0) ),
            ("width"     , Value::U32(1920) ),
            ("height"    , Value::U32(1080) ),
            ("fullscreen", Value::Bool(true)),
        ]);
    
        let binding = ini::Properties::default();
        let ini = Ini::load_from_file("chess_titans_rtx.conf").unwrap_or_default();
        let user_config = ini.section::<String>(None).unwrap_or(&binding);

        let mut config: HashMap<String, Value> = Default::default();
        for (k, v) in default_config {

            let value_str: &str;
            let result = user_config.get(k);
            match result {
                Some(value) => value_str = value,
                None => { config.insert(k.to_owned(), v); continue; } // Fallback to default config
            }

            let value: Value;
            match v {
                Value::I32(i)   => value = Value::I32(value_str.parse::<i32>().unwrap_or(i)  ),
                Value::U32(i)   => value = Value::U32(value_str.parse::<u32>().unwrap_or(i)  ),
                Value::F32(i)   => value = Value::F32(value_str.parse::<f32>().unwrap_or(i)  ),
                Value::Bool(i) => value = Value::Bool(value_str.parse::<bool>().unwrap_or(i)),
                Value::String(_)     => value = Value::String(value_str.to_owned()),
            }

            config.insert(k.to_string(), value);
        }

        config
    }
}