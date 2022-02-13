<template>
  <v-form
      ref="form"
      v-model="valid"
      lazy-validation
      :disabled="disabled"
      width="100%"
  >
    <v-col
        cols="24"
    >
      <label for="download-url">FoundryVTT Timed URL</label>
      <v-text-field
          id="download-url"
          v-model="foundryDownloadUrl"
          label=""
          required
          :rules="foundryDownloadUrlRules"
          placeholder="https://foundry"
          width="100%"
      />
      <v-btn
          :disabled="disabled"
          color="success"
          class="mr-4"
          @click="sendToExpress"
      >
        Submit
      </v-btn>
    </v-col>
  </v-form>
</template>
<style lang="scss">
#download-url {
  width: 18em;
  color: black;
}
</style>
<script>
import axios from "axios";
import useAlert, {Status} from "../composables/useAlert";

const alert = useAlert();

export default {
  data: () => ({
    valid: true,
    foundryDownloadUrl: '',
    foundryDownloadUrlRules: [
      v => !!v || 'Foundry URL is required',
      v => (v && v.includes("https://foundryvtt.s3.amazonaws.com")) || 'Url Must be from Foundry VTT',
    ],
    disabled: false,
    waitForFoundryInterval: null,

  }),
  beforeDestroy() {
    clearInterval(this.waitForFoundryInterval)
  },
  methods: {
    async killExpressServer() {
      return axios.post('/exit')
    },
    async waitForFoundry() {
      await new Promise(resolve => {
        this.waitForFoundryInterval = setInterval(()=> {
          axios.get('/license').then(({statusText}) => {
            if (statusText.toString().toLowerCase() === 'ok') {
              resolve(true)
            }
          } )
        }, 2500);
      })
      clearInterval(this.waitForFoundryInterval)
    },
    async sendToExpress () {
        this.disabled = true;
        alert.show(Status.Waiting)
        const uploadResponse = await axios.post('/uploader', {"foundry": this.foundryDownloadUrl});
        if (uploadResponse.status === 200) {
          await this.killExpressServer();
          await this.waitForFoundry();
          alert.show(Status.Success)
          setTimeout(()=> {
            window.location.href = '/license'
          }, 2000)
        } else {
          this.disabled = false;
          alert.show(Status.Failure)
        }
        this.disabled = false;
    },
  },
}
</script>
