fn main() {
    env_logger::init();

    let name = "example/sample/scripts/sample.toml";
    let mut comp = imuc_comp::comp::CompInst::new(
        imuc_comp::comp::config::Comp::read_file(std::path::Path::new(name)).unwrap(),
    );
    if comp.compile().is_some() {
        log::info!("Compilation done");
    } else {
        log::error!("Compilation has errors!");
    }
}
