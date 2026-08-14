// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pragma solidity ^0.8.30;

import {SquareVerifier} from "./fixtures/SquareVerifier.sol";

contract SquareVerifierTest {
    uint256 internal constant SCALAR_MODULUS = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001;

    SquareVerifier internal verifier;

    function setUp() public {
        verifier = new SquareVerifier();
    }

    function testValidRustProof() public view {
        uint256[] memory inputs = _inputs(81);
        require(verifier.verifyProof(_validProof(), inputs), "valid proof rejected");
    }

    function testWrongPublicInputIsRejected() public view {
        uint256[] memory inputs = _inputs(82);
        require(!verifier.verifyProof(_validProof(), inputs), "wrong input accepted");
    }

    function testWellFormedWrongProofElementsAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        bytes memory proof = _validProof();
        _replace(proof, 0, _g1Generator());
        require(!verifier.verifyProof(proof, inputs), "wrong A accepted");

        proof = _validProof();
        _replace(proof, 128, _g2Generator());
        require(!verifier.verifyProof(proof, inputs), "wrong B accepted");

        proof = _validProof();
        _replace(proof, 384, _g1Generator());
        require(!verifier.verifyProof(proof, inputs), "wrong C accepted");
    }

    function testNonCanonicalPublicInputIsRejected() public view {
        uint256[] memory inputs = _inputs(SCALAR_MODULUS);
        require(!verifier.verifyProof(_validProof(), inputs), "non-canonical input accepted");
    }

    function testWrongPublicInputCountIsRejected() public view {
        uint256[] memory noInputs = new uint256[](0);
        uint256[] memory twoInputs = new uint256[](2);
        require(!verifier.verifyProof(_validProof(), noInputs), "empty inputs accepted");
        require(!verifier.verifyProof(_validProof(), twoInputs), "extra input accepted");
    }

    function testWrongProofLengthIsRejected() public view {
        uint256[] memory inputs = _inputs(81);
        require(!verifier.verifyProof(new bytes(511), inputs), "short proof accepted");
        require(!verifier.verifyProof(new bytes(513), inputs), "long proof accepted");
    }

    function testIdentityProofElementsAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        bytes memory proof = _validProof();
        _zero(proof, 0, 128);
        require(!verifier.verifyProof(proof, inputs), "identity A accepted");

        proof = _validProof();
        _zero(proof, 128, 256);
        require(!verifier.verifyProof(proof, inputs), "identity B accepted");

        proof = _validProof();
        _zero(proof, 384, 128);
        require(!verifier.verifyProof(proof, inputs), "identity C accepted");
    }

    function testMalformedPointIsRejected() public view {
        uint256[] memory inputs = _inputs(81);
        bytes memory proof = _validProof();
        proof[63] ^= bytes1(uint8(1));
        require(!verifier.verifyProof(proof, inputs), "malformed point accepted");
    }

    function testInvalidFieldEncodingsAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        uint256[8] memory coordinates = [uint256(0), 64, 128, 192, 256, 320, 384, 448];
        for (uint256 i = 0; i < coordinates.length; ++i) {
            bytes memory proof = _validProof();
            proof[coordinates[i]] = 0x01;
            require(!verifier.verifyProof(proof, inputs), "nonzero coordinate padding accepted");
        }
    }

    function testBaseFieldModulusCoordinatesAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        uint256[8] memory coordinates = [uint256(0), 64, 128, 192, 256, 320, 384, 448];
        for (uint256 i = 0; i < coordinates.length; ++i) {
            bytes memory proof = _validProof();
            _replace(proof, coordinates[i], _baseFieldModulus());
            require(!verifier.verifyProof(proof, inputs), "coordinate equal to p accepted");
        }
    }

    function testOffCurveProofElementsAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        bytes memory proof = _validProof();
        _zero(proof, 64, 64);
        require(!verifier.verifyProof(proof, inputs), "off-curve A accepted");

        proof = _validProof();
        _zero(proof, 256, 128);
        require(!verifier.verifyProof(proof, inputs), "off-curve B accepted");

        proof = _validProof();
        _zero(proof, 448, 64);
        require(!verifier.verifyProof(proof, inputs), "off-curve C accepted");
    }

    function testWrongSubgroupProofElementsAreRejected() public view {
        uint256[] memory inputs = _inputs(81);
        bytes memory proof = _validProof();
        _replace(proof, 0, _g1NotInSubgroup());
        require(!verifier.verifyProof(proof, inputs), "wrong-subgroup A accepted");

        proof = _validProof();
        _replace(proof, 128, _g2NotInSubgroup());
        require(!verifier.verifyProof(proof, inputs), "wrong-subgroup B accepted");

        proof = _validProof();
        _replace(proof, 384, _g1NotInSubgroup());
        require(!verifier.verifyProof(proof, inputs), "wrong-subgroup C accepted");
    }

    function testEip2537PrecompileValidationAssumptions() public view {
        require(_pairingCallSucceeds(_g1Generator(), _g2Generator()), "valid points rejected");
        require(!_pairingCallSucceeds(_g1NotInSubgroup(), _g2Generator()), "G1 subgroup check missing");
        require(!_pairingCallSucceeds(_g1Generator(), _g2NotInSubgroup()), "G2 subgroup check missing");

        bytes memory invalidG1 = _g1Generator();
        _replace(invalidG1, 0, _baseFieldModulus());
        require(!_pairingCallSucceeds(invalidG1, _g2Generator()), "G1 field check missing");

        invalidG1 = _g1Generator();
        _zero(invalidG1, 64, 64);
        require(!_pairingCallSucceeds(invalidG1, _g2Generator()), "G1 curve check missing");

        bytes memory invalidG2 = _g2Generator();
        _zero(invalidG2, 128, 128);
        require(!_pairingCallSucceeds(_g1Generator(), invalidG2), "G2 curve check missing");
    }

    function _inputs(uint256 value) private pure returns (uint256[] memory inputs) {
        inputs = new uint256[](1);
        inputs[0] = value;
    }

    function _zero(bytes memory value, uint256 start, uint256 length) private pure {
        for (uint256 i = start; i < start + length; ++i) {
            value[i] = 0;
        }
    }

    function _replace(bytes memory value, uint256 start, bytes memory replacement) private pure {
        for (uint256 i = 0; i < replacement.length; ++i) {
            value[start + i] = replacement[i];
        }
    }

    function _pairingCallSucceeds(bytes memory g1, bytes memory g2) private view returns (bool success) {
        bytes memory input = bytes.concat(g1, g2);
        assembly ("memory-safe") {
            let callSuccess := staticcall(70300, 0x0f, add(input, 0x20), 0x180, 0, 0x20)
            success := and(callSuccess, eq(returndatasize(), 0x20))
        }
    }

    function _baseFieldModulus() private pure returns (bytes memory) {
        return hex"000000000000000000000000000000001a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
    }

    function _g1Generator() private pure returns (bytes memory) {
        return hex"0000000000000000000000000000000017f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb0000000000000000000000000000000008b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1";
    }

    function _g2Generator() private pure returns (bytes memory) {
        return hex"00000000000000000000000000000000024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb80000000000000000000000000000000013e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e000000000000000000000000000000000ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801000000000000000000000000000000000606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be";
    }

    // From Ethereum execution-spec-tests' EIP-2537 pairing failure vectors.
    function _g1NotInSubgroup() private pure returns (bytes memory) {
        return hex"000000000000000000000000000000000123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00000000000000000000000000000000193fb7cedb32b2c3adc06ec11a96bc0d661869316f5e4a577a9f7c179593987beb4fb2ee424dbb2f5dd891e228b46c4a";
    }

    // From Ethereum execution-spec-tests' EIP-2537 pairing failure vectors.
    function _g2NotInSubgroup() private pure returns (bytes memory) {
        return hex"00000000000000000000000000000000197bfd0342bbc8bee2beced2f173e1a87be576379b343e93232d6cef98d84b1d696e5612ff283ce2cfdccb2cfb65fa0c00000000000000000000000000000000184e811f55e6f9d84d77d2f79102fd7ea7422f4759df5bf7f6331d550245e3f1bcf6a30e3b29110d85e0ca16f9f6ae7a000000000000000000000000000000000f10e1eb3c1e53d2ad9cf2d398b2dc22c5842fab0a74b174f691a7e914975da3564d835cd7d2982815b8ac57f507348f000000000000000000000000000000000767d1c453890f1b9110fda82f5815c27281aba3f026ee868e4176a0654feea41a96575e0c4d58a14dbfbcc05b5010b1";
    }

    function _validProof() private pure returns (bytes memory) {
        return hex"0000000000000000000000000000000000bc6f9b7489b9c96981f75ae1b941437582ab781187909126f728fcfca5d5da1b1fbe9a679debbfed3a82dc6160f88700000000000000000000000000000000130908296327c3cb5a24cb63a5c358569a0515a996c4d946cc0f551559a0ae1ee51b246063b728ef95bc1341c358dac100000000000000000000000000000000039ddb9a6b61946888ec999e7e901c9f3abbb4590278bdbeb5cd7a7bf14f729293a8d2cfb4ae12ef127eebcf16d4dd4d0000000000000000000000000000000000cb7e5607018fdbe475f8c5183d9dea188894209d1180da0a6bfe5588c89cea12fedda054c51ce21ccd82090730946a00000000000000000000000000000000021c0ba9db15a0b0453199b2b9c2b767a6f0af2996bfb5ab2c74e976fba1105ac81b2cf6d73dde3abce326430ea30bd60000000000000000000000000000000003f5d87e82b6a4e5236a2d0bac0bc9c409a394827c0e9915ce87eaa3adb4e3a1a13ebd850abea3669920e2a83c3d1067000000000000000000000000000000000b79b74af6d7f3c6933b7e260e5780ec2278bcaad56dc52f9933f3007b8644b83df63ae949b2ab39eb15d1e2647a31ba00000000000000000000000000000000011d26c95ec1b1de25930b1e834fb53e138c1ddedda26f3c9db0b6e7a1ea496f9c65fdfb4c4df60474545cb942da26d2";
    }
}
