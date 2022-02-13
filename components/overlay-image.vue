<template>
  <div class="text-center">
    <v-img
        :src="src"
        @click="overlay = !overlay"
    />

    <v-overlay
        :value="overlay"
        class="align-center justify-center"
        @click="overlay = !overlay">
      <v-img
          class="align-center justify-center"
          width="200%"
          :src="src"
          @click="overlay = !overlay"
      />
    </v-overlay>
  </div>
</template>
<script>

export default {
  props: ['src'],

  data: () => ({
    overlay: false,
  }),
  mounted() {
    console.log(this.src)
  },

  // watch: {
  //   overlay (val) {
  //     val && setTimeout(() => {
  //       this.overlay = false
  //     }, 2000)
  //   },
  // },
}
</script>
