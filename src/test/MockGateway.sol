// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

contract MockGateway {
    event MockCall(bytes recipient, address zrc20, bytes message);
    event MockDeposit(address recipient, uint256 amount);
    
    function call(
        bytes memory recipient,
        address zrc20,
        bytes memory message,
        CallOptions memory callOptions,
        RevertOptions memory revertOptions
    ) external payable {
        emit MockCall(recipient, zrc20, message);
    }
    
    function deposit(address recipient) external payable {
        emit MockDeposit(recipient, msg.value);
    }
    
    function zetaToken() external pure returns (address) {
        return address(0x5FC8d32690cc91D4c39d9d3abcBD16989F875707); // Mock WZETA
    }
}

struct CallOptions {
    uint256 gasLimit;
    bool isArbitraryCall;
}

struct RevertOptions {
    address revertAddress;
    bool callOnRevert;
    address abortAddress;
    bytes revertMessage;
    uint256 onRevertGasLimit;
}
