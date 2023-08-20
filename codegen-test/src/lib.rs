include!(concat!(env!("OUT_DIR"), "/asyncapi.rs"));

#[cfg(test)]
mod test {
    use super::*;

    fn test_get_user_struct() {
        GetUser {};
    }
}
