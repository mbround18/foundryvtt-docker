import fs from "fs";
import path from "path";
import express from "express";
import fileUpload from "express-fileupload";
import { downloadTimedUrl } from "./utils/downloadTimedUrl";
import { deleteFiles } from "./utils/deleteFiles";

const app = express();
const port = 4444;
const data_dir = process.env.DATA_DIR || path.join(__dirname, "..", "tmp");
const foundryvttZipPath =
  process.env.FOUNDRYVTT_TMP_PATH || path.join(data_dir, "foundryvtt.zip");

const timedLinkRegex = /https\W+foundryvtt.+releases.+AWSAccessKeyId.+Signature.+Expires.+/;

app.use(
  fileUpload({
    useTempFiles: true,
    tempFileDir: path.join(data_dir, "tmp_uploads"),
  })
);

app.get("/uploader", (req, res) =>
  res.sendFile(path.join(__dirname, "index.html"))
);

app.get("/success", (req, res) => {
  res.sendFile(path.join(__dirname, "success.html"));
});

app.post("/uploader", async (req, res) => {
  const timedLink = req.body && req.body["foundryvtt-link"];
  if (!timedLinkRegex.test(timedLink)) {
    return res.send("The link your sent does not match expected!");
  }

  if (req.files && req.files["foundryvtt-link"]) {
    deleteFiles(Object.values(req.files));
  }

  if (timedLink) {
    await downloadTimedUrl({
      url: timedLink,
      destinationPath: foundryvttZipPath,
    });

    res.send("Completed upload! Starting services...");
    return process.exit(0);
  }
});

app.listen(port, "0.0.0.0", () => {
  console.log(`Please visit http://localhost:${port}/uploader\n`);
});
