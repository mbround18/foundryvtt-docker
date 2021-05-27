<template>
  <div>
    <v-alert
      :value="alertActive"
      :color="alertColor"
      dark
      border="top"
      transition="scale-transition"
    >{{ alertMessage }}</v-alert>
    <installation></installation>
  </div>

</template>

<script>
import Installation from "../components/installation";
export default {
  computed: {
    alertActive() {
      return this.$store.state.upload.alertActive
    },
    alertColor() {
      const colors = {
        success:'success',
        failure: 'failure',
        none: 'purple'
      }
      return colors[this.$store.state.upload.status] || 'warning'
    },
    alertMessage()  {
      const {status, statusMessages: messages} = this.$store.state.upload;
      return messages[status] || 'Unknown State'
    },
    // alertMessage ()  {
    //   const {statusMessages: messages, status} = this.$store.state.upload;
    //   return messages[status] || 'Unknown State!'
    // },
  },
  components: {Installation},
}
</script>
