import fs from "fs";
import shell from "shelljs";
import { validURL } from "./validUrl";

export async function downloadTimedUrl({ url, destinationPath }) {
  if (validURL(url) && url.includes("https://foundryvtt.s3.amazonaws.com")) {
    return shell.exec(`wget -nv --output-document=${destinationPath} "${url}"`);
  }
}
