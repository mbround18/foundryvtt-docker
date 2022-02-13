import * as path from "path"
import os from "os";
import shelljs from 'shelljs';


const __dirname = path.resolve(os.tmpdir());
export const DATA_DIR = process.env.DATA_DIR || path.join(__dirname, "downloads");
export const FOUNDRY_VTT_ZIP_PATH =
    process.env.FOUNDRYVTT_TMP_PATH || path.join(DATA_DIR, "foundryvtt.zip");


shelljs.mkdir('-p', DATA_DIR)
