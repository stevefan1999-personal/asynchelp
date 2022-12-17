use tarpc::context;

#[test]
fn test_tarpc_stackfuture_basic() {
    #[tarpc::service]
    trait Hello {
        async fn hello() -> String;
    }

    impl Hello for () {
        #[asynchelp::tarpc::stackfuture(size = 1024)]
        async fn hello(self, _: context::Context) -> String {
            "hello".to_string()
        }
    }
}

