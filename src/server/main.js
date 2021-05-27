import fs from "fs";
import path from "path";
import express from "express";
import fileUpload from "express-fileupload";
import { downloadTimedUrl } from "./utils/downloadTimedUrl";
import rateLimit from "express-rate-limit";
import shelljs from 'shelljs';

const app = express();
const port = 4444;
const data_dir = process.env.DATA_DIR || path.join(__dirname, "../..", "tmp");
const foundryvttZipPath =
  process.env.FOUNDRYVTT_TMP_PATH || path.join(data_dir, "foundryvtt.zip");

const timedLinkRegex = /https\W+foundryvtt.+releases.+AWSAccessKeyId.+Signature.+Expires.+/;
// cop,

const limiter = rateLimit({
  windowMs: 15 * 60 * 100, // 15 minutes
  max: 1000 // limit each IP to 100 requests per windowMs
});


shelljs.mkdir('-p', data_dir)

app.use(express.json())

//  apply to all requests
app.use(limiter);
app.use(express.static(path.join(__dirname, "../client")))

app.use(
  fileUpload({
    useTempFiles: true,
    tempFileDir: path.join(data_dir, "tmp_uploads"),
  })
);

app.get("/", (req, res) =>
  res.sendFile(path.resolve(path.join(__dirname, "../client/index.html")))
);

app.post('/exit', ((req, res) => {
  res.send(`success`);
  setTimeout(()=> {
    process.exit(0)
  }, 2000)
}))

app.post("/uploader", async (req, res) => {
  const timedLink = req.body && req.body["foundry"];
  console.log('Timed link received : ', {timedLink})
  if (!timedLinkRegex.test(timedLink)) {
    res.status(400)
    return res.send("The link your sent does not match expected!");
  }

  console.log('Timed link is valid : ', {timedLink, valid: true})

  // if (req.files && req.files["foundryvtt-link"]) {
  //   deleteFiles(Object.values(req.files));
  // }

  if (timedLink) {
    console.log('Downloading Timed link : ', {timedLink, valid: true, status: 'downloading'})
    await downloadTimedUrl({
      url: timedLink,
      destinationPath: foundryvttZipPath,
    });
    console.log('Downloading Timed link : ', {timedLink, valid: true, status: 'downloaded'})
    return res.send({status: 'success'})
  }
  console.log('Posted Information: ', {timedLink})
  res.status(500)
  return res.send(`Something went wrong!`)
});

app.listen(port, "0.0.0.0", () => {
  console.log(`Please visit http://localhost:${port}/\n`);
});
