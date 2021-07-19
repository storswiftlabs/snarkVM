// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    account::{ACCOUNT_COMMITMENT_INPUT, ACCOUNT_ENCRYPTION_INPUT, ACCOUNT_SIGNATURE_INPUT},
    testnet1::{
        outer_circuit_verifier_input::OuterCircuitVerifierInput,
        program::ProgramLocalData,
        transaction::Transaction,
        Testnet1Components,
        DPC,
    },
    DPCComponents,
    InnerCircuitVerifierInput,
    Network,
};
use snarkvm_algorithms::{
    commitment::{Blake2sCommitment, PedersenCompressedCommitment},
    crh::BoweHopwoodPedersenCompressedCRH,
    define_merkle_tree_parameters,
    encryption::GroupEncryption,
    prelude::*,
    prf::Blake2s,
    signature::Schnorr,
    snark::{gm17::GM17, groth16::Groth16},
};
use snarkvm_curves::{
    bls12_377::Bls12_377,
    bw6_761::BW6_761,
    edwards_bls12::{EdwardsParameters, EdwardsProjective as EdwardsBls12},
    edwards_bw6::EdwardsProjective as EdwardsBW6,
    PairingEngine,
};
use snarkvm_gadgets::{
    algorithms::{
        commitment::{Blake2sCommitmentGadget, PedersenCompressedCommitmentGadget},
        crh::BoweHopwoodPedersenCompressedCRHGadget,
        encryption::GroupEncryptionGadget,
        prf::Blake2sGadget,
        signature::SchnorrGadget,
        snark::{GM17VerifierGadget, Groth16VerifierGadget},
    },
    curves::{bls12_377::PairingGadget, edwards_bls12::EdwardsBls12Gadget, edwards_bw6::EdwardsBW6Gadget},
};

use once_cell::sync::OnceCell;

macro_rules! dpc_setup {
    ($fn_name: ident, $static_name: ident, $type_name: ident, $setup_msg: expr) => {
        #[inline]
        fn $fn_name() -> &'static Self::$type_name {
            static $static_name: OnceCell<<Testnet1Parameters as DPCComponents>::$type_name> = OnceCell::new();
            $static_name.get_or_init(|| Self::$type_name::setup($setup_msg))
        }
    };
}

pub type Testnet1DPC = DPC<Testnet1Parameters>;
pub type Testnet1Transaction = Transaction<Testnet1Parameters>;

define_merkle_tree_parameters!(
    CommitmentMerkleTreeParameters,
    <Testnet1Parameters as DPCComponents>::RecordCommitmentTreeCRH,
    32
);

pub struct Testnet1Parameters;

#[rustfmt::skip]
impl DPCComponents for Testnet1Parameters {
    const NETWORK_ID: u8 = Network::Testnet1.id();

    const NUM_INPUT_RECORDS: usize = 2;
    const NUM_OUTPUT_RECORDS: usize = 2;

    type InnerCurve = Bls12_377;
    type OuterCurve = BW6_761;

    type InnerScalarField = <Self::InnerCurve as PairingEngine>::Fr;
    type OuterScalarField = <Self::OuterCurve as PairingEngine>::Fr;
    type OuterBaseField = <Self::OuterCurve as PairingEngine>::Fq;

    type AccountCommitmentScheme = PedersenCompressedCommitment<EdwardsBls12, 8, 192>;
    type AccountCommitmentGadget = PedersenCompressedCommitmentGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 8, 192>;
    type AccountCommitment = <Self::AccountCommitmentScheme as CommitmentScheme>::Output;

    type AccountEncryption = GroupEncryption<EdwardsBls12>;
    type AccountEncryptionGadget = GroupEncryptionGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget>;

    type AccountSignature = Schnorr<EdwardsBls12>;
    type AccountSignatureGadget = SchnorrGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget>;
    type AccountSignaturePublicKey = <Self::AccountSignature as SignatureScheme>::PublicKey;

    type EncryptedRecordCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBls12, 48, 44>;
    type EncryptedRecordCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 48, 44>;
    type EncryptedRecordDigest = <Self::EncryptedRecordCRH as CRH>::Output;

    type EncryptionGroup = EdwardsBls12;
    type EncryptionGroupGadget = EdwardsBls12Gadget;
    type EncryptionParameters = EdwardsParameters;

    type InnerCircuitIDCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBW6, 304, 64>;
    type InnerCircuitIDCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBW6, Self::OuterScalarField, EdwardsBW6Gadget, 304, 64>;

    type LocalDataCommitmentScheme = PedersenCompressedCommitment<EdwardsBls12, 8, 162>;
    type LocalDataCommitmentGadget = PedersenCompressedCommitmentGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 8, 162>;

    type LocalDataCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBls12, 16, 32>;
    type LocalDataCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 16, 32>;
    type LocalDataDigest = <Self::LocalDataCRH as CRH>::Output;

    type PRF = Blake2s;
    type PRFGadget = Blake2sGadget;

    type ProgramCommitmentScheme = Blake2sCommitment;
    type ProgramCommitmentGadget = Blake2sCommitmentGadget;
    type ProgramCommitment = <Self::ProgramCommitmentScheme as CommitmentScheme>::Output;

    type ProgramIDCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBW6, 144, 63>;
    type ProgramIDCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBW6, Self::OuterScalarField, EdwardsBW6Gadget, 144, 63>;

    type RecordCommitmentScheme = PedersenCompressedCommitment<EdwardsBls12, 8, 233>;
    type RecordCommitmentGadget = PedersenCompressedCommitmentGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 8, 233>;
    type RecordCommitment = <Self::RecordCommitmentScheme as CommitmentScheme>::Output;

    type RecordCommitmentTreeCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBls12, 8, 32>;
    type RecordCommitmentTreeCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 8, 32>;
    type RecordCommitmentTreeDigest = <Self::RecordCommitmentTreeCRH as CRH>::Output;
    type RecordCommitmentTreeParameters = CommitmentMerkleTreeParameters;

    type SerialNumberNonceCRH = BoweHopwoodPedersenCompressedCRH<EdwardsBls12, 32, 63>;
    type SerialNumberNonceCRHGadget = BoweHopwoodPedersenCompressedCRHGadget<EdwardsBls12, Self::InnerScalarField, EdwardsBls12Gadget, 32, 63>;

    dpc_setup!{account_commitment, ACCOUNT_COMMITMENT, AccountCommitmentScheme, ACCOUNT_COMMITMENT_INPUT}
    dpc_setup!{account_encryption, ACCOUNT_ENCRYPTION, AccountEncryption, ACCOUNT_ENCRYPTION_INPUT}
    dpc_setup!{account_signature, ACCOUNT_SIGNATURE, AccountSignature, ACCOUNT_SIGNATURE_INPUT}
    dpc_setup!{encrypted_record_crh, ENCRYPTED_RECORD_CRH, EncryptedRecordCRH, "AleoEncryptedRecordCRH0"}
    dpc_setup!{inner_circuit_id_crh, INNER_CIRCUIT_ID_CRH, InnerCircuitIDCRH, "AleoInnerCircuitIDCRH0"}
    dpc_setup!{local_data_commitment, LOCAL_DATA_COMMITMENT, LocalDataCommitmentScheme, "AleoLocalDataCommitment0"}
    dpc_setup!{local_data_crh, LOCAL_DATA_CRH, LocalDataCRH, "AleoLocalDataCRH0"}
    dpc_setup!{program_id_commitment, PROGRAM_ID_COMMITMENT, ProgramCommitmentScheme, "AleoProgramIDCommitment0"}
    dpc_setup!{program_id_crh, PROGRAM_ID_CRH, ProgramIDCRH, "AleoProgramIDCRH0"}
    dpc_setup!{record_commitment, RECORD_COMMITMENT, RecordCommitmentScheme, "AleoRecordCommitment0"}
    dpc_setup!{record_commitment_tree_crh, RECORD_COMMITMENT_TREE_CRH, RecordCommitmentTreeCRH, "AleoRecordCommitmentTreeCRH0"}
    dpc_setup!{serial_number_nonce_crh, SERIAL_NUMBER_NONCE_CRH, SerialNumberNonceCRH, "AleoSerialNumberNonceCRH0"}

    // TODO (howardwu): TEMPORARY - Refactor this to a proper tree.
    fn record_commitment_tree_parameters() -> &'static Self::RecordCommitmentTreeParameters {
        static RECORD_COMMITMENT_TREE_PARAMETERS: OnceCell<<Testnet1Parameters as DPCComponents>::RecordCommitmentTreeParameters> = OnceCell::new();
        RECORD_COMMITMENT_TREE_PARAMETERS.get_or_init(|| Self::RecordCommitmentTreeParameters::from(Self::record_commitment_tree_crh().clone()))
    }
}

impl Testnet1Components for Testnet1Parameters {
    type InnerSNARK = Groth16<Self::InnerCurve, InnerCircuitVerifierInput<Testnet1Parameters>>;
    type InnerSNARKGadget = Groth16VerifierGadget<Self::InnerCurve, PairingGadget>;
    type NoopProgramSNARK = GM17<Self::InnerCurve, ProgramLocalData<Self>>;
    type NoopProgramSNARKGadget = GM17VerifierGadget<Self::InnerCurve, PairingGadget>;
    type OuterSNARK = Groth16<Self::OuterCurve, OuterCircuitVerifierInput<Testnet1Parameters>>;
}
