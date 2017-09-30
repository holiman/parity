// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

/// json_tracer.rs file is duplicated at evmbin/src/display/json.rs
/// copied here to get around external crate and module problem (cyclic dependency)

use std::collections::HashMap;
//use util::{U256, H256, ToPretty};
use bigint::prelude::U256;
use bigint::hash::H256;
use bytes::ToPretty;
use evm;
use trace::VMTracer;
use trace::trace::VMTrace;

/// Converts U256 into string.
/// TODO Overcomes: https://github.com/paritytech/bigint/issues/13
pub fn u256_as_str(v: &U256) -> String {
	if v.is_zero() {
		"\"0x0\"".into()
	} else {
		format!("\"{:x}\"", v)
	}
}

/// JSON formatting VM tracer.
#[derive(Default, Debug)]
pub struct JsonVMTracer {
	code: Vec<u8>,
	depth: usize,
	pc: usize,
	instruction: u8,
	gas_cost: U256,
	gas_used: U256,
	stack: Vec<U256>,
	memory: Vec<u8>,
	storage: HashMap<H256, H256>,
}

impl JsonVMTracer {
	fn memory(&self) -> String {
		format!("\"0x{}\"", self.memory.to_hex())
	}

	fn stack(&self) -> String {
		let items = self.stack.iter().map(u256_as_str).collect::<Vec<_>>();
		format!("[{}]", items.join(","))
	}

	fn storage(&self) -> String {
		let vals = self.storage.iter()
			.map(|(k, v)| format!("\"0x{:?}\": \"0x{:?}\"", k, v))
			.collect::<Vec<_>>();
		format!("{{{}}}", vals.join(","))
	}
}

impl VMTracer for JsonVMTracer {
	fn trace_next_instruction(&mut self, pc: usize, instruction: u8) -> bool {
		self.pc = pc;
		self.instruction = instruction;
		true
	}

	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, gas_cost: U256, gas_used: U256) {
		self.pc = pc;
		self.instruction = instruction;
		self.gas_cost = gas_cost;
		let info = evm::INSTRUCTIONS[self.instruction as usize];
		
		/*
		println!(
			"{{\"pc\":{pc},\"op\":{op},\"opName\":\"{name}\",\"gas\":{gas},\"memory\":{memory},\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = self.pc,
			op = self.instruction,
			name = info.name,
			gas = u256_as_str(&(gas_used)),
			memory = self.memory(),
			stack = self.stack(),
			storage = self.storage(),
			depth = self.depth,
		);
		*/
		
		println!(
			"{{\"pc\":{pc},\"op\":{op},\"opName\":\"{name}\",\"gas\":{gas},\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = self.pc,
			op = self.instruction,
			name = info.name,
			gas = u256_as_str(&(gas_used)),
			// memory = self.memory(),
			stack = self.stack(),
			storage = self.storage(),
			depth = self.depth,
		);
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		let info = evm::INSTRUCTIONS[self.instruction as usize];
		self.gas_used = gas_used;
		let len = self.stack.len();
		self.stack.truncate(len - info.args);
		self.stack.extend_from_slice(stack_push);

		if let Some((pos, data)) = mem_diff {
			if self.memory.len() < (pos + data.len()) {
				self.memory.resize((pos + data.len()), 0);
			}
			self.memory[pos..(pos + data.len())].copy_from_slice(data);
		}

		if let Some((pos, val)) = store_diff {
			self.storage.insert(pos.into(), val.into());
		}
	}

	fn prepare_subtrace(&self, code: &[u8]) -> Self where Self: Sized {
		let mut vm = JsonVMTracer::default();
		vm.depth = self.depth + 1;
		vm.code = code.to_vec();
		vm.gas_used = self.gas_used;
		vm
	}

	fn done_subtrace(&mut self, mut sub: Self) {
		if sub.depth == 1 {
			// print last line with final state:
			sub.gas_cost = 0.into();
			let gas_used = sub.gas_used;
			VMTracer::trace_executed(&mut sub, gas_used, &[], None, None);
		}
	}

	fn drain(self) -> Option<VMTrace> { None }
}
