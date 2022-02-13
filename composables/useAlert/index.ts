import { ref } from 'vue'

export enum Status {
    Warning = "Warning",
    Success = "Success",
    Failure = "Failure",
    Waiting = "Waiting",
    Unknown = "Unknown"
}

export default function () {
    const active = ref(false);
    const visible = ref(false);
    const status = ref(Status.Success);
    const messages = {
        [Status.Unknown]: 'You should not see this.',
        [Status.Waiting]: 'We are waiting for FoundryVTT to initialize...',
        [Status.Success]: 'Yay! FoundryVTT is online! Redirecting you now!',
        [Status.Failure]: 'Fuck shit! Something blew up! check logs...'
    }
    const colors = {
        [Status.Success]: "success",
        [Status.Failure]: "failure",
        [Status.Unknown]: "purple",
        [Status.Warning]: "orange",
        [Status.Waiting]: "blue"
    }

    function message() {
        return messages[status.value]
    }

    function color() {
        return colors[status.value]
    }

    function show(s?: Status) {
        active.value = true;
        status.value = s ?? Status.Unknown;
    }

    function hide() {
        active.value = false;
    }

    return {
        active,
        visible,
        message,
        color,
        hide,
        show
    }
};
