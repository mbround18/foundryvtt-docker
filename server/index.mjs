import { createServer } from 'http';
import { createApp } from 'h3'
import uploader from "./api/uploader.mjs";
import exit from "./api/exit.mjs";
import path from "path"
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename)

// const isDevelopment = (process.env.NODE_ENV ?? "development").toLowerCase() === "development"
const dist_dir = process.env.FRONTEND_DIR ? path.resolve(process.env.FRONTEND_DIR) : path.join(__dirname,"./dist/frontend")

import serveStatic from "serve-static";
import {fileURLToPath} from "url";

const app = createApp()
const port = process.env.PORT || 3000;

const serve = serveStatic(dist_dir, { index: ['index.html', 'index.htm'] })

app.use('/', serve)
app.use('/uploader', uploader)
app.use('/exit', exit)


console.log(`Listening on http://127.0.0.1:${port}`)
createServer(app).listen(port, '0.0.0.0')
