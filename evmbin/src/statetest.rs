extern crate ethcore;
extern crate ethjson;
extern crate ethcore_logger;

/// statest.rs file is duplicated in ethcore/src/json_tests/state.rs
/// copied here to get around problems that prevented using it as an external module

pub use util::*;

use ethcore::pod_state::PodState;
use ethcore::transaction::SignedTransaction;
use vm::EnvInfo;
use bigint::hash::H256;
use ethcore::trace::JsonVMTracer;
use ethcore::client::{EvmTestClient, EvmTestError, TransactResult};

pub fn json_chain_test(json_data: &[u8]) -> Vec<String> {
	ethcore_logger::init_log();
	let tests = ethjson::state::test::Test::load(json_data).unwrap();
	let mut failed = Vec::new();

	for (name, test) in tests.into_iter() {
		{
			let multitransaction = test.transaction;
			let env: EnvInfo = test.env.into();
			let pre: PodState = test.pre_state.into();

			for (spec_name, states) in test.post_states {
				println!("evmbin/src/statetest.rs json_chain_test for spec_name: {:?}", spec_name);
				let total = states.len();

				let spec = match EvmTestClient::spec_from_json(&spec_name) {
					Some(spec) => spec,
					None => {
						println!("   - {} | {:?} Ignoring tests because of missing spec", name, spec_name);
						continue;
					}
				};

				for (i, state) in states.into_iter().enumerate() {
					let info = format!("   - {} | {:?} ({}/{}) ...", name, spec_name, i + 1, total);

					let post_root: H256 = state.hash.into();
					let transaction: SignedTransaction = multitransaction.select(&state.indexes).into();

					let result = || -> Result<_, EvmTestError> {
						Ok(EvmTestClient::from_pod_state(spec, pre.clone())?
							.transact(&env, transaction, JsonVMTracer::default()))
					};
					match result() {
						Err(err) => {
							println!("{} !!! Unexpected internal error: {:?}", info, err);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Ok { state_root, .. }) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Err { state_root, ref error }) if state_root != post_root => {
							println!("{} !!! State mismatch (got: {}, expect: {}", info, state_root, post_root);
							println!("{} !!! Execution error: {:?}", info, error);
							flushln!("{} fail", info);
							failed.push(name.clone());
						},
						Ok(TransactResult::Err { error, .. }) => {
							flushln!("{} ok ({:?})", info, error);
						},
						Ok(TransactResult::Ok { state_root, .. }) => {
							println!("{{\"stateRoot\": \"{:?}\"}}", state_root);
							// TODO: println!("dumping state: {:?}", state);
							flushln!("{} ok", info);
						},
					}

				}
			}
		}

	}

	if !failed.is_empty() {
		println!("!!! {:?} tests failed.", failed.len());
	}
	failed
}

pub fn run_json_test(json_data: String) -> Vec<String> {
  json_chain_test(json_data.as_bytes())
}

