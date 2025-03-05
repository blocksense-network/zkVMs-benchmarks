/// Inserts array sizes before every square bracket
///
/// # Example
///
/// If `flat` is "[[0,1], [2,3,4], []]"
/// Output will be "3[2[0,1], 3[2,3,4], 0[]]"
pub fn get_with_sizes(flat: &str) -> String {
    let mut values = flat.split('[').map(|x| x.trim()).skip(1);
    let current = values.next().unwrap_or(flat);

    // 1D collection or not a collection
    if current != "" {
        let size = 1 + current
            .clone()
            .to_string()
            .chars()
            .take_while(|x| *x != ']')
            .map(|x| (x == ',') as usize)
            .sum::<usize>();

        (if size > 1 {
            size.to_string()
        } else {
            String::new()
        }) + "["
            + current
            + &values.map(|x| "[".to_string() + x).collect::<String>()
    }
    // ND collection
    else {
        let size: usize = values.clone().count();

        let subcollections = values.map(|x| get_with_sizes(x)).collect::<String>();

        size.to_string() + "[" + &subcollections
    }
}

#[macro_export]
macro_rules! to_bytes {
    ($($arg:tt)+) => {
        {
            // Simplify input string
            let flat = format!("{:?}", $($arg)+)
                .replace("false", "0")
                .replace("true",  "1")
                .replace('(', "[")
                .replace(')', "]")
                .replace('{', "[")
                .replace('}', "]");

            let flat = tobytes::get_with_sizes(&flat);

            flat
                .replace('[', ",")
                .replace(']', " ")
                .replace(':', ",")
                .split(',')
                .map(|val| {
                    let val = val.trim();
                    if let Some(num) = val.parse::<u64>().ok() {
                        vec![num]
                    }
                    else {
                        let val = val.trim_matches('"');
                        let mut size = vec![val.len() as u64];
                        size.extend(val
                            .bytes()
                            .into_iter()
                            .map(|x| x as u64)
                            .collect::<Vec<u64>>());
                        size
                    }
                })
                .flatten()
                .collect::<Vec<u64>>()
        }
    }
}
