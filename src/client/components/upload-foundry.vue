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
      <v-text-field
          v-model="foundryDownloadUrl"
          label="FoundryVTT Timed URL"
          required
          :rules="foundryDownloadUrlRules"
          placeholder="https://foundry"
          width="100%"
      ></v-text-field>
    </v-col>
    <v-btn
        :disabled="disabled"
        color="success"
        class="mr-4"
        @click="sendToExpress"
    >
      Submit
    </v-btn>
  </v-form>
</template>
<script>
import axios from "axios";

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
          // deepcode ignore PromiseNotCaughtGeneral: Iz all gucci we dont care if this errors out
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
      if (this.$refs.form.validate()) {
        this.disabled = true;
        this.$store.commit('upload/showAlert', 'pending')
        const uploadResponse = await axios.post('/uploader', {"foundry": this.foundryDownloadUrl});
        if (uploadResponse.status === 200) {
          await this.killExpressServer();
          await this.waitForFoundry();
          this.$store.commit('upload/showAlert', 'success')
          setTimeout(()=> {
            window.location.href = '/license'
          }, 2000)
        } else {
          this.disabled = false;
          this.$store.commit('upload/showAlert', 'failure')
        }
      } else {
        this.disabled = false;
      }
    },
  },
}
</script>
