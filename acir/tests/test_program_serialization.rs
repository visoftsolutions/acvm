//! This integration test defines a set of circuits which are used in order to test the acvm_js package.
//!
//! The acvm_js test suite contains serialized [circuits][`Circuit`] which must be kept in sync with the format
//! outputted from the [ACIR crate][acir].
//! Breaking changes to the serialization format then require refreshing acvm_js's test suite.
//! This file contains Rust definitions of these circuits and outputs the updated serialized format.
//!
//! These tests also check this circuit serialization against an expected value, erroring if the serialization changes.
//! Generally in this situation we just need to refresh the `expected_serialization` variables to match the
//! actual output, **HOWEVER** note that this results in a breaking change to the ACIR format.

use std::collections::BTreeSet;

use acir::{
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
        Circuit, Opcode, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::FieldElement;
use base64::Engine;
use brillig::{HeapArray, RegisterIndex, RegisterOrMemory};

#[test]
fn addition_circuit() {
    let addition = Opcode::Arithmetic(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(1)),
            (FieldElement::one(), Witness(2)),
            (-FieldElement::one(), Witness(3)),
        ],
        q_c: FieldElement::zero(),
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![addition],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs([Witness(3)].into()),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249, 100, 32, 27,
        219, 96, 119, 89, 37, 40, 176, 255, 8, 81, 36, 23, 72, 41, 195, 53, 215, 61, 221, 189, 35,
        132, 16, 195, 55, 217, 251, 244, 134, 127, 193, 184, 145, 149, 22, 22, 65, 101, 30, 173,
        12, 36, 188, 160, 88, 87, 1, 150, 94, 21, 21, 69, 229, 46, 74, 52, 148, 181, 89, 183, 6,
        134, 76, 3, 167, 24, 77, 135, 229, 125, 187, 32, 57, 231, 253, 154, 22, 151, 113, 113, 250,
        0, 123, 50, 20, 220, 112, 1, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn fixed_base_scalar_mul_circuit() {
    let fixed_base_scalar_mul = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::FixedBaseScalarMul {
        low: FunctionInput { witness: Witness(1), num_bits: 128 },
        high: FunctionInput { witness: Witness(2), num_bits: 128 },
        outputs: (Witness(3), Witness(4)),
    });

    let circuit = Circuit {
        current_witness_index: 5,
        opcodes: vec![fixed_base_scalar_mul],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(3), Witness(4)])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 138, 91, 10, 0, 48, 12, 194, 178, 215, 207, 78, 189,
        163, 175, 165, 10, 21, 36, 10, 57, 192, 160, 146, 188, 226, 139, 78, 113, 69, 183, 190, 61,
        111, 218, 182, 247, 1, 253, 52, 57, 128, 84, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn pedersen_circuit() {
    let pedersen = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Pedersen {
        inputs: vec![FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() }],
        outputs: (Witness(2), Witness(3)),
        domain_separator: 0,
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![pedersen],
        private_parameters: BTreeSet::from([Witness(1)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 65, 10, 0, 64, 8, 2, 103, 183, 232, 255, 47,
        142, 138, 58, 68, 130, 168, 140, 10, 60, 90, 149, 118, 182, 79, 255, 105, 57, 140, 197,
        246, 39, 0, 246, 174, 71, 87, 84, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn schnorr_verify_circuit() {
    let public_key_x =
        FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() };
    let public_key_y =
        FunctionInput { witness: Witness(2), num_bits: FieldElement::max_num_bits() };
    let signature =
        (3..(3 + 64)).map(|i| FunctionInput { witness: Witness(i), num_bits: 8 }).collect();
    let message = ((3 + 64)..(3 + 64 + 10))
        .map(|i| FunctionInput { witness: Witness(i), num_bits: 8 })
        .collect();
    let output = Witness(3 + 64 + 10);
    let last_input = output.witness_index() - 1;

    let schnorr = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
        public_key_x,
        public_key_y,
        signature,
        message,
        output,
    });

    let circuit = Circuit {
        current_witness_index: 100,
        opcodes: vec![schnorr],
        private_parameters: BTreeSet::from_iter((1..=last_input).map(Witness)),
        return_values: PublicInputs(BTreeSet::from([output])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 233, 50, 66, 1, 24, 199, 225, 99, 223, 247,
        125, 15, 73, 146, 36, 73, 146, 36, 73, 194, 93, 184, 255, 75, 48, 122, 167, 167, 25, 103,
        230, 204, 83, 211, 151, 230, 253, 255, 126, 146, 36, 25, 73, 6, 79, 56, 193, 223, 254, 59,
        202, 166, 223, 199, 250, 239, 116, 255, 29, 231, 4, 39, 57, 197, 225, 59, 195, 89, 206,
        113, 158, 11, 92, 228, 18, 151, 185, 194, 85, 174, 113, 157, 27, 220, 228, 22, 183, 185,
        195, 93, 238, 113, 159, 7, 60, 228, 17, 83, 60, 230, 9, 79, 153, 230, 25, 51, 60, 103, 150,
        23, 204, 241, 146, 121, 94, 177, 192, 107, 22, 121, 195, 18, 111, 89, 230, 29, 43, 188,
        103, 149, 15, 172, 241, 145, 117, 62, 177, 193, 103, 54, 249, 194, 214, 191, 29, 227, 121,
        245, 189, 205, 55, 118, 248, 206, 46, 63, 216, 227, 39, 191, 248, 237, 115, 60, 209, 94,
        116, 23, 173, 69, 103, 209, 88, 244, 53, 108, 107, 198, 255, 136, 150, 162, 163, 104, 40,
        250, 137, 118, 162, 155, 104, 38, 122, 137, 86, 162, 147, 104, 36, 250, 136, 54, 162, 139,
        104, 34, 122, 136, 22, 162, 131, 104, 32, 246, 143, 237, 83, 201, 96, 243, 216, 59, 182,
        78, 219, 56, 99, 219, 172, 77, 115, 182, 204, 219, 176, 96, 187, 162, 205, 74, 182, 42,
        219, 168, 98, 155, 170, 77, 106, 182, 168, 219, 160, 225, 246, 77, 55, 111, 185, 113, 219,
        109, 59, 110, 218, 117, 203, 158, 27, 14, 111, 54, 188, 91, 226, 150, 127, 214, 93, 14,
        165, 212, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn simple_brillig_foreign_call() {
    let w_input = Witness(1);
    let w_inverted = Witness(2);

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(w_input.into()), // Input Register 0,
        ],
        // This tells the BrilligSolver which witnesses its output registers correspond to
        outputs: vec![
            BrilligOutputs::Simple(w_inverted), // Output Register 1
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![brillig::Opcode::ForeignCall {
            function: "invert".into(),
            destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
        }],
        predicate: None,
    };

    let opcodes = vec![Opcode::Brillig(brillig_data)];
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 65, 10, 0, 32, 8, 4, 205, 32, 122, 142, 253,
        160, 207, 116, 232, 210, 33, 162, 247, 23, 100, 96, 32, 93, 106, 64, 92, 92, 144, 93, 15,
        0, 6, 22, 86, 104, 201, 190, 69, 222, 244, 70, 48, 255, 126, 145, 204, 139, 74, 102, 63,
        199, 177, 206, 165, 167, 218, 110, 13, 15, 80, 152, 168, 248, 3, 190, 43, 105, 200, 59, 1,
        0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn complex_brillig_foreign_call() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let a = Witness(1);
    let b = Witness(2);
    let c = Witness(3);

    let a_times_2 = Witness(4);
    let b_times_3 = Witness(5);
    let c_times_4 = Witness(6);
    let a_plus_b_plus_c = Witness(7);
    let a_plus_b_plus_c_times_2 = Witness(8);

    let brillig_data = Brillig {
        inputs: vec![
            // Input Register 0
            BrilligInputs::Array(vec![
                Expression::from(a),
                Expression::from(b),
                Expression::from(c),
            ]),
            // Input Register 1
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, a), (fe_1, b), (fe_1, c)],
                q_c: fe_0,
            }),
        ],
        // This tells the BrilligSolver which witnesses its output registers correspond to
        outputs: vec![
            BrilligOutputs::Array(vec![a_times_2, b_times_3, c_times_4]), // Output Register 0
            BrilligOutputs::Simple(a_plus_b_plus_c),                      // Output Register 1
            BrilligOutputs::Simple(a_plus_b_plus_c_times_2),              // Output Register 2
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            // Oracles are named 'foreign calls' in brillig
            brillig::Opcode::ForeignCall {
                function: "complex".into(),
                inputs: vec![
                    RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
                ],
                destinations: vec![
                    RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(2)),
                ],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![Opcode::Brillig(brillig_data)];
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 219, 10, 128, 48, 8, 245, 210, 101, 159, 179,
        254, 160, 127, 137, 222, 138, 122, 236, 243, 91, 228, 64, 44, 232, 33, 7, 117, 64, 156,
        206, 201, 193, 51, 3, 0, 32, 156, 224, 100, 36, 103, 148, 88, 35, 215, 245, 226, 227, 59,
        116, 232, 215, 43, 150, 226, 72, 63, 224, 200, 5, 56, 230, 255, 240, 81, 189, 61, 117, 113,
        157, 31, 223, 236, 79, 149, 172, 78, 214, 72, 220, 138, 15, 106, 214, 168, 114, 249, 126,
        88, 230, 117, 26, 55, 54, 37, 90, 26, 155, 39, 227, 31, 223, 232, 230, 4, 215, 157, 63,
        176, 3, 89, 64, 134, 157, 36, 4, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn memory_op_circuit() {
    let init = vec![Witness(1), Witness(2)];

    let memory_init = Opcode::MemoryInit { block_id: BlockId(0), init };
    let write = Opcode::MemoryOp {
        block_id: BlockId(0),
        op: MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
        predicate: None,
    };
    let read = Opcode::MemoryOp {
        block_id: BlockId(0),
        op: MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
        predicate: None,
    };

    let circuit = Circuit {
        current_witness_index: 5,
        opcodes: vec![memory_init, write, read],
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        return_values: PublicInputs([Witness(4)].into()),
        ..Circuit::default()
    };
    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 146, 49, 14, 0, 32, 8, 3, 171, 192, 127, 240, 7,
        254, 255, 85, 134, 136, 9, 131, 155, 48, 216, 165, 76, 165, 92, 16, 0, 132, 45, 113, 239,
        238, 205, 103, 198, 93, 211, 93, 223, 52, 170, 115, 91, 114, 46, 229, 101, 105, 53, 92,
        253, 13, 46, 103, 222, 78, 161, 164, 125, 50, 5, 16, 167, 184, 45, 92, 160, 252, 96, 232,
        6, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn deserialize_from_nargo() {
    const BYTECODE: &str = "H4sIAAAAAAAA/7WTMRLEIAhFMYkp9ywgGrHbq6yz5v5H2JkdCyaxC9LgWDw+H9gBwMM91p7fPeOzIKdYjEeMLYdGTB8MpUrCmOohJJQkfYMwN4mSSy0ZC0VudKbCZ4cthqzVrsc/yw28dMZeWmrWerfBexnsxD6hJ7jUufr4GvyZFp8xpG0C14Pd8s/q29vPCBXypvmpDx7sD8opnfqIfsM1RNtxBQAA";
    let circuit_bytes_compressed =
        base64::engine::general_purpose::STANDARD.decode(BYTECODE).unwrap();
    let circuit = Circuit::read(circuit_bytes_compressed.as_slice()).unwrap();
    println!("{:?}", circuit)
}
