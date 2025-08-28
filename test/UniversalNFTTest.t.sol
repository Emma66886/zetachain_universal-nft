// SPDX-License-Identifier: MIT
pragma solidity ^0.8.3;

import "forge-std/Test.sol";
import "../contracts/UniversalNFT.sol";
import "../contracts/ConnectedNFT.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract UniversalNFTTest is Test {
    UniversalNFT public universalNFT;
    ConnectedNFT public connectedNFT;
    
    address public owner = address(0x1);
    address public user = address(0x2);
    address public gateway = address(0x3);
    address public uniswapRouter = address(0x4);
    address public zrc20 = address(0x5);
    
    uint256 public constant GAS_LIMIT = 500000;

    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy implementation
        UniversalNFT implementation = new UniversalNFT();
        
        // Deploy proxy
        bytes memory initData = abi.encodeWithSelector(
            UniversalNFT.initialize.selector,
            owner,
            "Universal NFT",
            "UNFT",
            payable(gateway),
            GAS_LIMIT,
            uniswapRouter
        );
        
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        universalNFT = UniversalNFT(address(proxy));
        
        // Deploy connected NFT contract
        connectedNFT = new ConnectedNFT(
            payable(gateway),
            "Connected NFT",
            "CNFT"
        );
        
        vm.stopPrank();
    }

    function testMintNFT() public {
        vm.startPrank(owner);
        
        string memory tokenURI = "https://example.com/token/1";
        universalNFT.safeMint(user, tokenURI);
        
        // Check that NFT was minted
        assertEq(universalNFT.balanceOf(user), 1);
        assertEq(universalNFT.ownerOf(1), user);
        assertEq(universalNFT.tokenURI(1), tokenURI);
        
        vm.stopPrank();
    }

    function testSetConnected() public {
        vm.startPrank(owner);
        
        bytes memory contractAddress = abi.encodePacked(address(connectedNFT));
        universalNFT.setConnected(zrc20, contractAddress);
        
        // Check that connection was set
        assertEq(universalNFT.connected(zrc20), contractAddress);
        
        vm.stopPrank();
    }

    function testBurnNFT() public {
        vm.startPrank(owner);
        
        // First mint an NFT
        string memory tokenURI = "https://example.com/token/1";
        universalNFT.safeMint(user, tokenURI);
        uint256 tokenId = 1;
        
        vm.stopPrank();
        vm.startPrank(user);
        
        // Burn the NFT
        universalNFT.burn(tokenId);
        
        // Check that NFT was burned
        assertEq(universalNFT.balanceOf(user), 0);
        vm.expectRevert();
        universalNFT.ownerOf(tokenId);
        
        vm.stopPrank();
    }

    function testConnectedNFTMint() public {
        vm.startPrank(owner);
        
        uint256 tokenId = 1;
        string memory tokenURI = "https://example.com/token/1";
        
        connectedNFT.mintFromCrossChain(user, tokenId, tokenURI);
        
        // Check that NFT was minted
        assertEq(connectedNFT.balanceOf(user), 1);
        assertEq(connectedNFT.ownerOf(tokenId), user);
        assertEq(connectedNFT.tokenURI(tokenId), tokenURI);
        assertTrue(connectedNFT.nftExists(tokenId));
        
        vm.stopPrank();
    }

    function testConnectedNFTBurn() public {
        vm.startPrank(owner);
        
        uint256 tokenId = 1;
        string memory tokenURI = "https://example.com/token/1";
        
        // First mint
        connectedNFT.mintFromCrossChain(user, tokenId, tokenURI);
        
        vm.stopPrank();
        vm.startPrank(user);
        
        // Then burn
        connectedNFT.burnForCrossChain(tokenId);
        
        // Check that NFT was burned
        assertEq(connectedNFT.balanceOf(user), 0);
        assertFalse(connectedNFT.nftExists(tokenId));
        vm.expectRevert();
        connectedNFT.ownerOf(tokenId);
        
        vm.stopPrank();
    }

    function testCannotMintSameTokenIdTwice() public {
        vm.startPrank(owner);
        
        uint256 tokenId = 1;
        string memory tokenURI = "https://example.com/token/1";
        
        connectedNFT.mintFromCrossChain(user, tokenId, tokenURI);
        
        // Try to mint same token ID again
        vm.expectRevert(ConnectedNFT.NFTAlreadyExists.selector);
        connectedNFT.mintFromCrossChain(user, tokenId, tokenURI);
        
        vm.stopPrank();
    }

    function testCannotBurnNonExistentNFT() public {
        vm.startPrank(user);
        
        uint256 tokenId = 999;
        
        vm.expectRevert(ConnectedNFT.NFTDoesNotExist.selector);
        connectedNFT.burnForCrossChain(tokenId);
        
        vm.stopPrank();
    }

    function testUnauthorizedMint() public {
        vm.startPrank(user); // Not owner
        
        uint256 tokenId = 1;
        string memory tokenURI = "https://example.com/token/1";
        
        vm.expectRevert();
        connectedNFT.mintFromCrossChain(user, tokenId, tokenURI);
        
        vm.stopPrank();
    }

    function testPauseUnpause() public {
        vm.startPrank(owner);
        
        // Pause contract
        universalNFT.pause();
        
        // Try to mint while paused
        vm.expectRevert();
        universalNFT.safeMint(user, "https://example.com/token/1");
        
        // Unpause
        universalNFT.unpause();
        
        // Should work now
        universalNFT.safeMint(user, "https://example.com/token/1");
        assertEq(universalNFT.balanceOf(user), 1);
        
        vm.stopPrank();
    }

    function testSupportsInterface() public {
        // Test ERC165 interface support
        assertTrue(universalNFT.supportsInterface(0x01ffc9a7)); // ERC165
        assertTrue(universalNFT.supportsInterface(0x80ac58cd)); // ERC721
        assertTrue(universalNFT.supportsInterface(0x5b5e139f)); // ERC721Metadata
        assertTrue(universalNFT.supportsInterface(0x780e9d63)); // ERC721Enumerable
    }

    // Test receiving ETH
    function testReceiveETH() public {
        vm.deal(address(this), 1 ether);
        
        (bool success, ) = address(universalNFT).call{value: 0.1 ether}("");
        assertTrue(success);
        
        assertEq(address(universalNFT).balance, 0.1 ether);
    }
}
