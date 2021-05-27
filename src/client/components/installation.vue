<template>
  <v-container class="d-flex flex-row flex-wrap " width="100%">
        <v-card
            elevation="10"
            width="25em"
            min-height="20em"
            v-for="(step, i) in steps"
            :key="JSON.stringify(step)"
            class="ma-5"
        >
          <v-card-title>
            Step {{i+1}}: {{step.name}}
          </v-card-title>
          <overlay-image v-if="step.src" :src="step.src"/>
          <v-card-text v-if="step.description">
            <p class="wrap">{{step.description}}</p>
          </v-card-text>
          <v-card-text v-if="step.instructions">
            <v-list>
              <v-list-item v-for="(instruction, i) in step.instructions" :key="JSON.stringify(instruction)">
                <p class="pr-1">{{ String.fromCharCode(i + 'A'.charCodeAt(0)) }})</p>
                <p class="wrap d-inline" v-html="instruction"></p>
              </v-list-item>
            </v-list>
          </v-card-text>
          <v-card-actions v-if="step.component">
            <component :is="step.component"></component>
          </v-card-actions>
        </v-card>
  </v-container>
</template>

<script>
import OverlayImage from "./overlay-image";
import UploadFoundry from "./upload-foundry";
export default {
  components: {UploadFoundry, OverlayImage},
  data: () => ({
    model: 0,
    steps: [
      {name: "Welcome!", description: "Hey there! Lets get FoundryVTT installed! :)", src: "https://placekitten.com/g/601/300"},
      {
        name: "Obtaining The Link",
        instructions: [
            "Open <a href='https://foundryvtt.com/' target='_blank'>foundryvtt.com</a>.",
            "Log into your Foundry account.",
            "Click your profile name once logged in."
        ],
        src: require("../assets/foundryvtt-click-username.png")
      },
      {
        name: "Copy the timed NodeJS Link",
        instructions: [
            "On your profile page click <strong>Purchased Licenses</strong>.",
            "(Optional) Change the version down.",
            "Change the Operating System to <strong>Linux/NodeJS</strong>.",
            "Click the <strong>Timed URL</strong> button."
        ],
        src: require('../assets/foundryvtt-click-timed-url.png')
      },
      {
        name: "Upload Foundry VTT",
        component: UploadFoundry,
        src: require('../assets/upload.png')
      }
    ],
  }),
}
</script>

