// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ConnectedNFT is ERC721, ERC721URIStorage, ERC721Burnable, Ownable {
    using SafeERC20 for IERC20;

    GatewayEVM public immutable gateway;

    // Mapping to track if an NFT exists (to prevent re-minting)
    mapping(uint256 => bool) public nftExists;
    
    // Events
    event NFTMintedFromCrossChain(address indexed receiver, uint256 indexed tokenId, string uri);
    event NFTBurnedForCrossChain(address indexed owner, uint256 indexed tokenId, string uri);
    event RevertEvent(string, RevertContext);

    error Unauthorized();
    error NFTAlreadyExists();
    error NFTDoesNotExist();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    constructor(
        address payable gatewayAddress,
        string memory name,
        string memory symbol
    ) ERC721(name, symbol) Ownable(msg.sender) {
        gateway = GatewayEVM(gatewayAddress);
    }

    /**
     * @notice Mint an NFT when received from cross-chain transfer
     */
    function mintFromCrossChain(
        address to,
        uint256 tokenId,
        string memory uri
    ) external onlyOwner {
        if (nftExists[tokenId]) revert NFTAlreadyExists();
        
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, uri);
        nftExists[tokenId] = true;

        emit NFTMintedFromCrossChain(to, tokenId, uri);
    }

    /**
     * @notice Burn an NFT for cross-chain transfer
     */
    function burnForCrossChain(uint256 tokenId) external {
        if (!nftExists[tokenId]) revert NFTDoesNotExist();
        if (ownerOf(tokenId) != msg.sender) revert Unauthorized();

        string memory uri = tokenURI(tokenId);
        _burn(tokenId);
        nftExists[tokenId] = false;

        emit NFTBurnedForCrossChain(msg.sender, tokenId, uri);
    }

    /**
     * @notice Handle incoming cross-chain calls from ZetaChain
     */
    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway {
        // Decode the cross-chain NFT transfer message
        (
            bytes32 receiver,
            uint256 tokenId,
            string memory uri,
            uint256 gasAmount,
            address originalSender
        ) = abi.decode(message, (bytes32, uint256, string, uint256, address));

        // Convert bytes32 receiver to address (for EVM chains)
        address receiverAddress = address(uint160(uint256(receiver)));

        // Mint the NFT
        if (!nftExists[tokenId]) {
            _safeMint(receiverAddress, tokenId);
            _setTokenURI(tokenId, uri);
            nftExists[tokenId] = true;

            emit NFTMintedFromCrossChain(receiverAddress, tokenId, uri);
        }
    }

    /**
     * @notice Handle cross-chain call failures
     */
    function onRevert(RevertContext calldata revertContext) external onlyGateway {
        emit RevertEvent("ConnectedNFT call reverted", revertContext);
    }

    /**
     * @notice Call a contract on ZetaChain or another connected chain
     */
    function call(
        address receiver,
        bytes calldata message,
        RevertOptions memory revertOptions
    ) external {
        gateway.call(receiver, message, revertOptions);
    }

    /**
     * @notice Deposit and call a contract on ZetaChain
     */
    function deposit(
        address receiver,
        RevertOptions memory revertOptions
    ) external payable {
        gateway.deposit{value: msg.value}(receiver, revertOptions);
    }

    /**
     * @notice Deposit ERC20 tokens and call a contract on ZetaChain
     */
    function deposit(
        address receiver,
        uint256 amount,
        address asset,
        RevertOptions memory revertOptions
    ) external {
        IERC20(asset).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(asset).approve(address(gateway), amount);
        gateway.deposit(receiver, amount, asset, revertOptions);
    }

    // Override required functions

    function tokenURI(uint256 tokenId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    // Allow contract to receive Ether
    receive() external payable {}
}
