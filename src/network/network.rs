#[derive(Clone)]
pub struct Network<'a> {
    pub app: &'a Arc<Mutex<App>>,
    endpoint: &'a str,
    etherscan: &'a Option<Etherscan>,
}
