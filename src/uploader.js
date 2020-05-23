import fs from 'fs';
import path from 'path';
import express from 'express';
import fileUpload from 'express-fileupload';
import { downloadTimedUrl } from './utils/downloadTimedUrl';
import { deleteFiles } from './utils/deleteFiles';

const app = express();
const port = 4444;
const data_dir = process.env.DATA_DIR || path.join(__dirname, '..', 'tmp');
const adminPasswdPath = path.join(data_dir, '.admin-password')
const foundryvttZipPath = process.env.FOUNDRYVTT_TMP_PATH || path.join(data_dir, 'foundryvtt.zip')

app.use(fileUpload({
    useTempFiles : true,
    tempFileDir : path.join(data_dir, 'tmp_uploads')
}));


app.get('/uploader', (req, res) => res.sendFile(path.join(__dirname, 'index.html')));
app.post('/uploader', async (req, res) => {
    const savedSecret = fs.readFileSync(adminPasswdPath, {encoding: 'utf-8'});
    const sentSecret = req.body && req.body['admin-secret'];
    const timedLink = req.body && req.body['foundryvtt-link'];
    const foundryvttUpload = req.files && req.files['foundryvtt-file'] && req.files['foundryvtt-file'].tempFilePath;

    if (sentSecret !== savedSecret) {
        if (req.files && req.files['foundryvtt-link']) {
            deleteFiles(Object.values(req.files));
        }
        return res.status(403).send('Unauthorized');    
    }

    if (timedLink) {
        await downloadTimedUrl({
            url: timedLink, 
            destinationPath: foundryvttZipPath
        });
        res.send('Completed');
        return process.exit(0);
    }

    if (foundryvttUpload) {
        fs.copyFileSync(foundryvttUpload, foundryvttZipPath)
        deleteFiles(Object.values(req.files));
        res.send('Completed!!!');
        return process.exit(0);
    }   
})
app.listen(port, "0.0.0.0", () => {
    const adminPasswd = require('crypto').randomBytes(64).toString('hex');
    fs.writeFileSync(adminPasswdPath, adminPasswd);
    console.log(`Please visit http://localhost:${port}/uploader\nPlease Check here for the admin password: ${adminPasswdPath}`)
});
