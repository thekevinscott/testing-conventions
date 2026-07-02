fn main() {
    std::process::exit(agent_context_core::run(std::env::args().skip(1)));
}
