pub mod models {
    include!(concat!(env!("OUT_DIR"), "/asyncapi.rs"));
}

#[cfg(test)]
mod test {
    pub use crate::models::*;

    #[test]
    fn test() {
        let x: SampleRequestPayload;
    }
}
