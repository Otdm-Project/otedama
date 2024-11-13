use std::sync::Mutex;
use std::io::Result;

static DOMAIN: &str = "otdm.dev";
static ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

lazy_static::lazy_static! {
    static ref SUBDOMAIN_COUNTER: Mutex<Vec<usize>> = Mutex::new(vec![0]);
}

// サブドメイン生成
pub fn generate_subdomain() -> Result<String> {
    let mut counter = SUBDOMAIN_COUNTER.lock().unwrap();
    let mut subdomain = String::new();

    for &index in counter.iter() {
        subdomain.push(ALPHABET.chars().nth(index).unwrap());
    }

    counter[0] += 1;
    let mut i = 0;
    while i < counter.len() && counter[i] == ALPHABET.len() {
        counter[i] = 0;
        if i + 1 < counter.len() {
            counter[i + 1] += 1;
        } else {
            counter.push(0);
        }
        i += 1;
    }

    let full_domain = format!("{}.{}", subdomain, DOMAIN);
    println!("Generated subdomain: {}", full_domain);

    Ok(full_domain)
}