use super::hash_table::{HashTableChallenges, HashTableEndpoints};
use super::instruction_table::{InstructionTableChallenges, InstructionTableEndpoints};
use super::io_table::{IOTableChallenges, IOTableEndpoints};
use super::jump_stack_table::{JumpStackTableChallenges, JumpStackTableEndpoints};
use super::op_stack_table::{OpStackTableChallenges, OpStackTableEndpoints};
use super::processor_table::{ProcessorTableChallenges, ProcessorTableEndpoints};
use super::program_table::{ProgramTableChallenges, ProgramTableEndpoints};
use super::ram_table::{RamTableChallenges, RamTableEndpoints};
use super::u32_op_table::{U32OpTableChallenges, U32OpTableEndpoints};
use crate::shared_math::stark::triton::state::DIGEST_LEN;
use crate::shared_math::x_field_element::XFieldElement;

#[derive(Debug, Clone)]
pub struct AllChallenges {
    pub program_table_challenges: ProgramTableChallenges,
    pub instruction_table_challenges: InstructionTableChallenges,
    pub input_table_challenges: IOTableChallenges,
    pub output_table_challenges: IOTableChallenges,
    pub processor_table_challenges: ProcessorTableChallenges,
    pub op_stack_table_challenges: OpStackTableChallenges,
    pub ram_table_challenges: RamTableChallenges,
    pub jump_stack_table_challenges: JumpStackTableChallenges,
    pub hash_table_challenges: HashTableChallenges,
    pub u32_op_table_challenges: U32OpTableChallenges,
}

impl AllChallenges {
    pub const TOTAL: usize = 10;

    pub fn create_challenges(weights: &[XFieldElement]) -> Self {
        let mut weights = weights.to_vec();

        let program_table_challenges = ProgramTableChallenges {
            instruction_eval_row_weight: weights.pop().unwrap(),
            address_weight: weights.pop().unwrap(),
            instruction_weight: weights.pop().unwrap(),
        };

        let instruction_table_challenges = InstructionTableChallenges {
            processor_perm_row_weight: weights.pop().unwrap(),
            ip_weight: weights.pop().unwrap(),
            ci_processor_weight: weights.pop().unwrap(),
            nia_weight: weights.pop().unwrap(),
            program_eval_row_weight: weights.pop().unwrap(),
            addr_weight: weights.pop().unwrap(),
            instruction_weight: weights.pop().unwrap(),
        };

        let input_table_challenges = IOTableChallenges {
            processor_eval_row_weight: weights.pop().unwrap(),
        };

        let output_table_challenges = IOTableChallenges {
            processor_eval_row_weight: weights.pop().unwrap(),
        };

        let processor_table_challenges = ProcessorTableChallenges {
            input_table_eval_row_weight: weights.pop().unwrap(),
            output_table_eval_row_weight: weights.pop().unwrap(),
            to_hash_table_eval_row_weight: weights.pop().unwrap(),
            from_hash_table_eval_row_weight: weights.pop().unwrap(),
            instruction_perm_row_weight: weights.pop().unwrap(),
            op_stack_perm_row_weight: weights.pop().unwrap(),
            ram_perm_row_weight: weights.pop().unwrap(),
            jump_stack_perm_row_weight: weights.pop().unwrap(),
            u32_lt_perm_row_weight: weights.pop().unwrap(),
            u32_and_perm_row_weight: weights.pop().unwrap(),
            u32_xor_perm_row_weight: weights.pop().unwrap(),
            u32_reverse_perm_row_weight: weights.pop().unwrap(),
            u32_div_perm_row_weight: weights.pop().unwrap(),
            instruction_table_ip_weight: weights.pop().unwrap(),
            instruction_table_ci_processor_weight: weights.pop().unwrap(),
            instruction_table_nia_weight: weights.pop().unwrap(),
            op_stack_table_clk_weight: weights.pop().unwrap(),
            op_stack_table_ci_weight: weights.pop().unwrap(),
            op_stack_table_osv_weight: weights.pop().unwrap(),
            op_stack_table_osp_weight: weights.pop().unwrap(),
            ram_table_clk_weight: weights.pop().unwrap(),
            ram_table_ramv_weight: weights.pop().unwrap(),
            ram_table_ramp_weight: weights.pop().unwrap(),
            jump_stack_table_clk_weight: weights.pop().unwrap(),
            jump_stack_table_ci_weight: weights.pop().unwrap(),
            jump_stack_table_jsp_weight: weights.pop().unwrap(),
            jump_stack_table_jso_weight: weights.pop().unwrap(),
            jump_stack_table_jsd_weight: weights.pop().unwrap(),
            hash_table_stack_input_weights: weights
                .drain(0..2 * DIGEST_LEN)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            hash_table_digest_output_weights: weights
                .drain(0..DIGEST_LEN)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            u32_op_table_lt_lhs_weight: weights.pop().unwrap(),
            u32_op_table_lt_rhs_weight: weights.pop().unwrap(),
            u32_op_table_lt_result_weight: weights.pop().unwrap(),
            u32_op_table_and_lhs_weight: weights.pop().unwrap(),
            u32_op_table_and_rhs_weight: weights.pop().unwrap(),
            u32_op_table_and_result_weight: weights.pop().unwrap(),
            u32_op_table_xor_lhs_weight: weights.pop().unwrap(),
            u32_op_table_xor_rhs_weight: weights.pop().unwrap(),
            u32_op_table_xor_result_weight: weights.pop().unwrap(),
            u32_op_table_reverse_lhs_weight: weights.pop().unwrap(),
            u32_op_table_reverse_result_weight: weights.pop().unwrap(),
            u32_op_table_div_divisor_weight: weights.pop().unwrap(),
            u32_op_table_div_remainder_weight: weights.pop().unwrap(),
            u32_op_table_div_result_weight: weights.pop().unwrap(),
        };

        let op_stack_table_challenges = OpStackTableChallenges {
            processor_perm_row_weight: weights.pop().unwrap(),
            clk_weight: weights.pop().unwrap(),
            ci_weight: weights.pop().unwrap(),
            osv_weight: weights.pop().unwrap(),
            osp_weight: weights.pop().unwrap(),
        };

        let ram_table_challenges = RamTableChallenges {
            processor_perm_row_weight: weights.pop().unwrap(),
            clk_weight: weights.pop().unwrap(),
            ramv_weight: weights.pop().unwrap(),
            ramp_weight: weights.pop().unwrap(),
        };

        let jump_stack_table_challenges = JumpStackTableChallenges {
            processor_perm_row_weight: weights.pop().unwrap(),
            clk_weight: weights.pop().unwrap(),
            ci_weight: weights.pop().unwrap(),
            jsp_weight: weights.pop().unwrap(),
            jso_weight: weights.pop().unwrap(),
            jsd_weight: weights.pop().unwrap(),
        };

        let stack_input_weights = weights
            .drain(0..2 * DIGEST_LEN)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let digest_output_weights = weights
            .drain(0..DIGEST_LEN)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let hash_table_challenges = HashTableChallenges {
            from_processor_eval_row_weight: weights.pop().unwrap(),
            to_processor_eval_row_weight: weights.pop().unwrap(),

            stack_input_weights,
            digest_output_weights,
        };

        let u32_op_table_challenges = U32OpTableChallenges {
            processor_lt_perm_row_weight: weights.pop().unwrap(),
            processor_and_perm_row_weight: weights.pop().unwrap(),
            processor_xor_perm_row_weight: weights.pop().unwrap(),
            processor_reverse_perm_row_weight: weights.pop().unwrap(),
            processor_div_perm_row_weight: weights.pop().unwrap(),
            lt_lhs_weight: weights.pop().unwrap(),
            lt_rhs_weight: weights.pop().unwrap(),
            lt_result_weight: weights.pop().unwrap(),
            and_lhs_weight: weights.pop().unwrap(),
            and_rhs_weight: weights.pop().unwrap(),
            and_result_weight: weights.pop().unwrap(),
            xor_lhs_weight: weights.pop().unwrap(),
            xor_rhs_weight: weights.pop().unwrap(),
            xor_result_weight: weights.pop().unwrap(),
            reverse_lhs_weight: weights.pop().unwrap(),
            reverse_result_weight: weights.pop().unwrap(),
            div_divisor_weight: weights.pop().unwrap(),
            div_remainder_weight: weights.pop().unwrap(),
            div_result_weight: weights.pop().unwrap(),
        };

        AllChallenges {
            program_table_challenges,
            instruction_table_challenges,
            input_table_challenges,
            output_table_challenges,
            processor_table_challenges,
            op_stack_table_challenges,
            ram_table_challenges,
            jump_stack_table_challenges,
            hash_table_challenges,
            u32_op_table_challenges,
        }
    }
}

/// An *endpoint* is the collective term for *initials* and *terminals*.
#[derive(Debug, Clone)]
pub struct AllEndpoints {
    pub program_table_endpoints: ProgramTableEndpoints,
    pub instruction_table_endpoints: InstructionTableEndpoints,
    pub input_table_endpoints: IOTableEndpoints,
    pub output_table_endpoints: IOTableEndpoints,
    pub processor_table_endpoints: ProcessorTableEndpoints,
    pub op_stack_table_endpoints: OpStackTableEndpoints,
    pub ram_table_endpoints: RamTableEndpoints,
    pub jump_stack_table_endpoints: JumpStackTableEndpoints,
    pub hash_table_endpoints: HashTableEndpoints,
    pub u32_op_table_endpoints: U32OpTableEndpoints,
}

impl AllEndpoints {
    pub const TOTAL: usize = 10;

    pub fn create_initials(weights: &[XFieldElement]) -> Self {
        let mut weights = weights.to_vec();

        let processor_table_initials = ProcessorTableEndpoints {
            input_table_eval_sum: weights.pop().unwrap(),
            output_table_eval_sum: weights.pop().unwrap(),
            instruction_table_perm_product: weights.pop().unwrap(),
            opstack_table_perm_product: weights.pop().unwrap(),
            ram_table_perm_product: weights.pop().unwrap(),
            jump_stack_perm_product: weights.pop().unwrap(),
            to_hash_table_eval_sum: weights.pop().unwrap(),
            from_hash_table_eval_sum: weights.pop().unwrap(),
            u32_table_lt_perm_product: weights.pop().unwrap(),
            u32_table_and_perm_product: weights.pop().unwrap(),
            u32_table_xor_perm_product: weights.pop().unwrap(),
            u32_table_reverse_perm_product: weights.pop().unwrap(),
            u32_table_div_perm_product: weights.pop().unwrap(),
        };

        let program_table_initials = ProgramTableEndpoints {
            instruction_eval_sum: weights.pop().unwrap(),
        };

        let instruction_table_initials = InstructionTableEndpoints {
            processor_perm_product: processor_table_initials.instruction_table_perm_product,
            program_eval_sum: program_table_initials.instruction_eval_sum,
        };

        let input_table_initials = IOTableEndpoints {
            processor_eval_sum: processor_table_initials.input_table_eval_sum,
        };

        let output_table_initials = IOTableEndpoints {
            processor_eval_sum: processor_table_initials.output_table_eval_sum,
        };

        let op_stack_table_initials = OpStackTableEndpoints {
            processor_perm_product: processor_table_initials.opstack_table_perm_product,
        };

        let ram_table_initials = RamTableEndpoints {
            processor_perm_product: processor_table_initials.ram_table_perm_product,
        };

        let jump_stack_table_initials = JumpStackTableEndpoints {
            processor_perm_product: processor_table_initials.jump_stack_perm_product,
        };

        // hash_table.from_processor <-> processor_table.to_hash, and
        // hash_table.to_processor   <-> processor_table.from_hash
        let hash_table_initials = HashTableEndpoints {
            from_processor_eval_sum: processor_table_initials.to_hash_table_eval_sum,
            to_processor_eval_sum: processor_table_initials.from_hash_table_eval_sum,
        };

        let u32_op_table_initials = U32OpTableEndpoints {
            processor_lt_perm_product: processor_table_initials.u32_table_lt_perm_product,
            processor_and_perm_product: processor_table_initials.u32_table_and_perm_product,
            processor_xor_perm_product: processor_table_initials.u32_table_xor_perm_product,
            processor_reverse_perm_product: processor_table_initials.u32_table_reverse_perm_product,
            processor_div_perm_product: processor_table_initials.u32_table_div_perm_product,
        };

        AllEndpoints {
            program_table_endpoints: program_table_initials,
            instruction_table_endpoints: instruction_table_initials,
            input_table_endpoints: input_table_initials,
            output_table_endpoints: output_table_initials,
            processor_table_endpoints: processor_table_initials,
            op_stack_table_endpoints: op_stack_table_initials,
            ram_table_endpoints: ram_table_initials,
            jump_stack_table_endpoints: jump_stack_table_initials,
            hash_table_endpoints: hash_table_initials,
            u32_op_table_endpoints: u32_op_table_initials,
        }
    }
}
