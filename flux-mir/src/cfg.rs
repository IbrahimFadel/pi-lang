use crate::mir::{Block, FnDecl, Instruction, MirID, RValue};

fn print_rval(rval: &RValue) -> String {
	let mut dot_str = String::new();

	match rval {
		RValue::I64(int) => dot_str += &int.to_string(),
		RValue::I32(int) => dot_str += &int.to_string(),
		RValue::I16(int) => dot_str += &int.to_string(),
		RValue::I8(int) => dot_str += &int.to_string(),
		RValue::U64(int) => dot_str += &int.to_string(),
		RValue::U32(int) => dot_str += &int.to_string(),
		RValue::U16(int) => dot_str += &int.to_string(),
		RValue::U8(int) => dot_str += &int.to_string(),
		RValue::F64(float) => dot_str += &float.to_string(),
		RValue::F32(float) => dot_str += &float.to_string(),
		RValue::Local(local) => dot_str += &format!("%{}", local),
	}

	return dot_str;
}

fn block_to_str(block: &Block, conns: &mut Vec<MirID>) -> String {
	let mut dot_str = String::new();
	for instr in &block.instrs {
		match instr {
			Instruction::Alloca(alloca) => {
				dot_str += &format!("%{} = alloca {}", alloca.id, alloca.ty);
			}
			Instruction::Store(store) => {
				dot_str += &format!(
					"%{} = store {} {} %{}",
					store.id,
					store.ty,
					print_rval(&store.val),
					store.ptr
				);
			}
			Instruction::Load(load) => {
				dot_str += &format!("%{} = load {} %{}", load.id, load.ty, load.ptr);
			}
			Instruction::Br(br) => {
				dot_str += &format!("br %block{}", br.to);
				conns.push(br.to);
			}
			Instruction::BrCond(brcond) => {
				dot_str += &format!(
					"brcond {} %block{} %block{}",
					print_rval(&brcond.cond),
					brcond.then,
					brcond.else_
				);
				conns.push(brcond.then);
				conns.push(brcond.else_);
			}
			Instruction::Ret(ret) => {
				if let Some(val) = &ret.val {
					dot_str += &format!("ret {}", print_rval(val));
				} else {
					dot_str += &format!("ret void");
				}
			}
			Instruction::Add(add) => {
				dot_str += &format!(
					"%{} = add {} {}",
					add.id,
					print_rval(&add.lhs),
					print_rval(&add.rhs)
				);
			}
			Instruction::CmpEq(cmpeq) => {
				dot_str += &format!(
					"%{} = cmp {} {} {}",
					cmpeq.id,
					cmpeq.ty,
					print_rval(&cmpeq.lhs),
					print_rval(&cmpeq.rhs)
				);
			}
			Instruction::Call(call) => {
				dot_str += &format!("%{} = call", call.id);
			}
			Instruction::IndexAccess(idx_access) => {
				dot_str += &format!(
					"%{} = idx %{} {}",
					idx_access.id, idx_access.ptr, idx_access.idx
				);
			}
			Instruction::PtrCast(ptr_cast) => {
				dot_str += &format!(
					"%{} = ptrcast %{} to {}",
					ptr_cast.id, ptr_cast.ptr, ptr_cast.to_ty
				);
			}
		};
		dot_str += "\\l";
	}
	return dot_str;
}

pub fn print_fn(f: &FnDecl) -> String {
	let mut dot_str = String::from("digraph ");
	dot_str += f.name.as_str();
	dot_str += " {\n\tsubgraph cluster {\n\t\t";
	dot_str += &format!(r#"label = "Fn: {}";"#, f.name.to_string());
	dot_str += "\n";
	let mut all_conns: Vec<(MirID, MirID)> = vec![];
	for block in &f.blocks {
		let mut conns: Vec<MirID> = vec![];
		dot_str += "\t\t";
		dot_str += &format!(
			r#""block{}" [shape = "record", label = "%block{}|{}"]"#,
			block.id,
			block.id,
			block_to_str(block, &mut conns)
		);
		dot_str += "\n";
		for conn in conns {
			all_conns.push((block.id, conn));
		}
	}

	dot_str += "\n";
	for (i, conn) in all_conns.iter().enumerate() {
		dot_str += "\t\t";
		dot_str += &format!(r#""block{}" -> "block{}""#, conn.0, conn.1);
		if i != all_conns.len() - 1 {
			dot_str += "\n";
		}
	}

	dot_str += "\n";
	dot_str += "\t}\n}\n";

	return dot_str;
}