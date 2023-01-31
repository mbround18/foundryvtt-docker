import * as fs from "fs";
import axios from "axios";
import { validUrl } from "./validUrl";

export async function downloadTimedUrl({ url, destinationPath }) {
  if (validUrl(url) && url.includes("https://foundryvtt.s3.amazonaws.com")) {
    console.log("Downloading foundry from : ", { url, destinationPath });
    return await axios({
      method: "get",
      url,
      responseType: "stream",
    })
      .then(function (response) {
        const stream = fs.createWriteStream(destinationPath, { flags: "w" });
        response.data.pipe(stream);
        console.log("Downloaded Foundry Zip to : ", { url, destinationPath });

        return new Promise((resolve) =>
          stream.on("finish", () => resolve(true))
        );
      })
      .catch(function (err) {
        console.log(err);
        process.exit(1);
      });
  }
  return false;
}
