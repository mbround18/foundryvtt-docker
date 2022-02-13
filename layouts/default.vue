<template>
  <v-app>
    <v-app-bar app>
      <welcome></welcome>
    </v-app-bar>

    <!-- Sizes your content based upon application components -->
    <v-main>
      <!-- Provides the application the proper gutter -->
      <v-container fluid>
        <slot />
      </v-container>
    </v-main>

    <v-footer app>
      <!-- -->
    </v-footer>
  </v-app>
</template>
<script>
import Welcome from "../components/welcome";
export default {
  components: { Welcome },
};
</script>

<style>
</style>
