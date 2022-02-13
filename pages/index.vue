<template>
  <div>
    <v-alert
      :hidden="alert.visible"
      :color="alert.color()"
      dark
      border="top"
      transition="scale-transition"
      >{{ alert.message() }}</v-alert
    >
    <installation></installation>
  </div>
</template>

<script setup>
import useAlert from "../composables/useAlert";

const alert = useAlert()
</script>

<script>
import Installation from "../components/installation";
export default {
  components: { Installation },
};
</script>
