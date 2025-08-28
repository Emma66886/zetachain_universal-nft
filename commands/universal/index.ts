import { Command } from "commander";

import { callCommand } from "./call";
import { withdrawCommand } from "./withdraw";
import { withdrawAndCallCommand } from "./withdrawAndCall";

// Import NFT commands
import "./mintNFT";
import "./transferNFT";
import "./setConnected";

export const universal = new Command("universal")
  .addCommand(callCommand)
  .addCommand(withdrawCommand)
  .addCommand(withdrawAndCallCommand);
