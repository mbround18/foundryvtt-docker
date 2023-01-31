import * as path from "path";
import os from "os";
import fs from "fs";

const __dirname = path.resolve(os.tmpdir());
export const DATA_DIR =
  process.env.DATA_DIR || path.join(__dirname, "downloads");
export const FOUNDRY_VTT_ZIP_PATH =
  process.env.FOUNDRYVTT_TMP_PATH || path.join(DATA_DIR, "foundryvtt.zip");

if (!fs.existsSync(DATA_DIR)) {
  fs.mkdirSync(DATA_DIR);
}
