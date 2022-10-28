

import express from "express"

import uploader from "./api/uploader.ts";
import exit from "./api/exit.ts";
import path from "path"
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename)

const dist_dir = process.env.FRONTEND_DIR ? path.resolve(process.env.FRONTEND_DIR) : path.join(__dirname,"./dist/frontend")
import {fileURLToPath} from "url";

const app = express()
const port = process.env.PORT || 3000;

app.use(express.json())

app.use('/', express.static(dist_dir, { index: ['index.html', 'index.htm'] }))
app.use('/uploader', uploader)
app.use('/exit', exit)


console.log(`Listening on http://127.0.0.1:${port}`)
app.listen(port, '0.0.0.0')
