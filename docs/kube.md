# Installation on Kubernetes

> I built my setup on a k3s stack. Please take a look at [k3s.io](https://k3s.io) for details on setting up k3s. Otherwise you may have to tweak your configuration a bit.

### Installation Steps

1. Add a namespace

   > \${PWD}/00-namespace.yml

   ```yaml
   ---
   apiVersion: v1
   kind: Namespace
   metadata:
     name: foundryvtt
     labels:
       name: foundryvtt
   ```

2. Set up a persisted volume claim.
   Note: You must already have a persisted volume storage configuration setup. In this example I am using [k3s.io](https://k3s.io) which already has a local storage persisted volume claim setup.

   > \${PWD}/01-persisted-volume-claim.yml

   ```yaml
   ---
   apiVersion: v1
   kind: PersistentVolumeClaim
   metadata:
     name: foundryvtt-data-pv-claim
     namespace: foundryvtt
   spec:
     storageClassName: local-path
     accessModes:
       - ReadWriteOnce
     resources:
       requests:
         storage: 40G
   ```

3. Setup the deployment, in this I have configured an SFTP server along side my foundry setup for file system access.
   Note 1: The known issue here is, after you first launch the SFTP server and log into it, you might have to format the permissions to allow reading and writting. I am currenly working on a solution for this.
   Note 2: With the SFTP server you will have to have a dedicated IP address on a node in its current configuration... (if you know a way to put it behind ingress and a domain name please DM me on discord.

   > \${PWD}/02-deployment.yml

   ```yaml
   ---
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     namespace: foundryvtt
     name: foundryvtt
     labels:
       app: foundryvtt

   spec:
     replicas: 1
     selector:
       matchLabels:
         app: foundryvtt
     template:
       metadata:
         labels:
           app: foundryvtt
       spec:
         containers:
           - name: foundryvtt-web
             image: mbround18/foundryvtt-docker:latest
             env:
               - name: APPLICATION_HOST
                 valueFrom:
                   secretKeyRef:
                     name: foundryvtt-env
                     key: APPLICATION_HOST
               - name: APPLICATION_PORT
                 value: "4444"
               - name: SSL_PROXY
                 value: "true"
             ports:
               - name: web
                 containerPort: 4444
             volumeMounts:
               - name: foundryvtt-data-persistent-storage
                 mountPath: /foundrydata
           - name: foundryvtt-ftp
             image: atmoz/sftp
             env:
               - name: FTP_USER
                 value: gm
               - name: FTP_PASS
                 valueFrom:
                   secretKeyRef:
                     name: foundryvtt-env
                     key: FTP_PASS
             command: []
             args: ["$(FTP_USER):$(FTP_PASS):1001:95687:foundryvtt-data"]
             ports:
               - name: sftp
                 containerPort: 22
             volumeMounts:
               - name: foundryvtt-data-persistent-storage
                 mountPath: /home/gm/foundryvtt-data
         volumes:
           - name: foundryvtt-data-persistent-storage
             persistentVolumeClaim:
               claimName: foundryvtt-data-pv-claim
   ```

   Note: In a folder called secrets I have placed the env file:

   > \${PWD}/secrets/env.txt

   ```txt
   APPLICATION_HOST=your.domain
   FTP_PASS=changeme
   ```

   You will have to update these accordingly to your setup.

4. Create the service file

   > \${PWD}/03-web-service.yml

   ```yaml
   ---
   apiVersion: v1
   kind: Service
   metadata:
     name: foundryvtt-web
     labels:
       name: foundryvtt-web
     namespace: foundryvtt
   spec:
     selector:
       app: foundryvtt
     ports:
       - name: web
         port: 80
         targetPort: 4444
   ---
   apiVersion: v1
   kind: Service
   metadata:
     name: foundryvtt-ftp
     labels:
       name: foundryvtt-ftp
     namespace: foundryvtt
   spec:
     type: NodePort
     selector:
       app: foundryvtt
     externalIPs:
       - xx.xx.xx.xx
     ports:
       - port: 2222
         targetPort: 22
   ```

5. Configure the Ingress for the web client
   Note: As a reminder, I am using [k3s.io](https://k3s.io) for my deployment. So when it comes to ingress, it ships with traefik and you might have to alter the configuration to match your setup.

   > \${PWD}/04-ingress.yml

   ```yaml
   ---
   apiVersion: extensions/v1beta1
   kind: Ingress
   metadata:
     name: foundryvtt
     namespace: foundryvtt
     annotations:
       kubernetes.io/ingress.class: traefik
   spec:
     rules:
       - host: your.domain
         http:
           paths:
             - backend:
                 serviceName: foundryvtt-web
                 servicePort: web
     tls:
       - secretName: foundryvtt-web-tls-cert
   ```

6. Lets link all this together with a `kustomization.yml`

   > \${PWD}/kustomization.yml

   ```yml
   ---
   apiVersion: kustomize.config.k8s.io/v1beta1
   kind: Kustomization
   namespace: foundryvtt

   commonLabels:
     app: foundryvtt

   resources:
     - 00-namespace.yml
     - 01-persisted-volume-claim.yml
     - 02-deployment.yml
     - 03-web-service.yml
     - 04-ingress.yml

   secretGenerator:
     - name: foundryvtt-env
       # env is a path to a file to read lines of key=val
       # you can only specify one env file per secret.
       env: secrets/env.txt
       type: Opaque
     - name: foundryvtt-web-tls-cert
       files:
         - secrets/tls.crt
         - secrets/tls.key
       type: "kubernetes.io/tls"
   ```

7. Before we execute, we need to generate TLS certificates. In my case I have a ruby script that generates certificates before deployment but for this example, you can generate your TLS secrets with the following command:

   ```sh
   openssl req –new –newkey rsa:4096 –nodes –keyout secrets/tls.key –out secrets/tls.crt
   ```

   Follow the prompts and if you have questions [Click Here for more info](https://www.digicert.com/kb/csr-ssl-installation/ubuntu-server-with-apache2-openssl.htm). Once completed this should create the following files:

   > ${PWD}/secrets/tls.key
   > ${PWD}/secrets/tls.crt

8. Open up CloudFlare or what ever you are using to configure your DNS Provider and setup a CNAME to point to your root domain. Or set up an A record to point to the IP address of your master node. In my case I have my base domain pointing to my master node, so that andy CNAME only has to point to the root domain. If this step is confusing, please [Click Here for more info](https://www.cloudflare.com/learning/dns/dns-records/dns-cname-record/).

9. Now its time to stand up the instance! :) Just run:

   ```sh
   kubectl apply -k .
   ```

10. Give it a couple minute, and your instance should be online :)
11. Follow post installation steps as needed.

### Post Installation (Kube)

Much like the post installation of a local setup, you will still have to upload the initial package. This can be accomplished by the following steps.

1. Navigate to `https://your.domain/uploader`
2. Open your profile on FoundryVTT
3. Navigate to your Purchased Licenses page
4. Click the link icon to get a timed link.
5. Paste that link on the uploader screen.
6. CLick the submit button.
7. If you get a Completed!! message, navigate to `https://your.domain/` and setup foundry as your normally would.
