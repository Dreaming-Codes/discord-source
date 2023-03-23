<script lang="ts" setup>
import {appWindow} from "@tauri-apps/api/window";
import {nextTick, onMounted, onUnmounted, reactive} from "vue";
import {watchArray} from "@vueuse/core";

interface Connection {
  source: BoundedElement,
  target: BoundedElement,
}

interface BoundedElement {
  element?: HTMLElement;
  connectionPoint: {
    x: number;
    y: number;
  };
}

const connections = reactive<Connection[]>([]);

const sources = reactive<number[]>([]);
const targets = reactive<number[]>([]);

watchArray([sources, targets], async () => {
  handleRedraw();
}, {
  flush: "post",
})

appWindow.listen("stream-added", (event) => {
  sources.push(event.payload as number);
})

appWindow.listen("stream-removed", (event) => {
  const index = sources.indexOf(event.payload as number);
  if (index > -1) {
    sources.splice(index, 1);
  }
})

appWindow.listen("web-added", (event) => {
  targets.push(event.payload as number);
})

appWindow.listen("web-removed", (event) => {
  const index = targets.indexOf(event.payload as number);
  if (index > -1) {
    targets.splice(index, 1);
  }
})

let hoveredElement: BoundedElement | null = null;

onMounted(() => {
  window.addEventListener("resize", handleRedraw)
})

onUnmounted(() => {
  window.removeEventListener("resize", handleRedraw)
})

async function handleRedraw() {
  console.log("Redrawing svg");
  await nextTick();
  connections.forEach((connection, index) => {
    if (!document.body.contains(connection.source.element!) || !document.body.contains(connection.target.element!)) {
      connections.splice(index, 1);
      return;
    }

    const sourceRect = connection.source.element?.getBoundingClientRect()!;
    connection.source.connectionPoint = {
      x: sourceRect.right,
      y: sourceRect.top + sourceRect.height / 2,
    };

    const targetRect = connection.target.element?.getBoundingClientRect()!;
    connection.target.connectionPoint = {
      x: targetRect.left,
      y: targetRect.top + targetRect.height / 2,
    };
  })
}

function mouseOver(e: MouseEvent) {
  const element = e.target as HTMLElement;
  const rect = element.getBoundingClientRect();
  hoveredElement = {
    element,
    connectionPoint: {
      x: rect.left,
      y: rect.top + rect.height / 2,
    }
  };
}

function mouseOut() {
  hoveredElement = null;
}

function startDrawing(e: DragEvent) {
  const targetImg = e.target as HTMLImageElement;
  const imgRect = targetImg.getBoundingClientRect();

  let currentLine = reactive<Connection>({
    source: {
      element: targetImg,
      connectionPoint: {
        x: imgRect.right,
        y: imgRect.top + imgRect.height / 2,
      }
    },
    target: {
      connectionPoint: {
        x: e.clientX,
        y: e.clientY,
      }
    }
  });
  connections.push(currentLine);

  function updateLine(e: MouseEvent) {
    if (hoveredElement) {
      currentLine.target = hoveredElement;
    } else {
      currentLine.target.connectionPoint = {
        x: e.clientX,
        y: e.clientY,
      };
    }
  }

  function stopDrawing() {
    if (!hoveredElement) {
      connections.pop();
    } else {
      //Find existing connection
      const existingConnection = connections.filter((connection) => connection.target.element === hoveredElement?.element && connection.source.element === currentLine.source.element);
      //Remove current and existing connection if they exist
      if (existingConnection.length > 1) {
        connections.splice(connections.indexOf(currentLine), 1);
        connections.splice(connections.indexOf(existingConnection[0]), 1);
      }
    }
    window.removeEventListener("mousemove", updateLine);
    window.removeEventListener("mouseup", stopDrawing);
  }

  window.addEventListener("mousemove", updateLine);
  window.addEventListener("mouseup", stopDrawing);
}

function imgLoad(){
  //TODO: Find a way that doesn't require a timeout
  //This is needed for now because vuetify doesn't update the DOM immediately after the image is loaded and using nextTick doesn't work
  setTimeout(handleRedraw, 100);
}
</script>

<template>
  <v-container fluid class="fill-height">
    <v-row class="d-flex justify-space-between">
      <v-col cols="4">
        <div>
          <v-img v-for="source in sources" :key="source" @load="imgLoad" @dragstart.prevent="startDrawing"
                 :src="'https://picsum.photos/1920/1080?' + source"></v-img>
        </div>
      </v-col>

      <v-col cols="4">
        <div>
          <v-img v-for="target in targets" :key="target" @load="imgLoad" @dragstart.prevent @mouseover="mouseOver" @mouseout="mouseOut"
                 :src="'https://picsum.photos/1920/1080?' + target"></v-img>
        </div>
      </v-col>
    </v-row>
    <svg id="lineDrawer" class="position-absolute fill-height w-100">
      <line v-for="line in connections" :x1="line.source.connectionPoint.x" :y1="line.source.connectionPoint.y"
            :x2="line.target.connectionPoint.x" :y2="line.target.connectionPoint.y"
            stroke-width="1" stroke="white"/>
    </svg>
  </v-container>
</template>

<style scoped lang="scss">
#lineDrawer {
  pointer-events: none;
  left: 0;
  top: 0;
}
</style>
