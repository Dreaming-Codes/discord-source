<script lang="ts" setup>
import {invoke} from "@tauri-apps/api/tauri";
import {onMounted, onUnmounted, reactive, ref} from "vue";

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

const sources = ref<string[]>(["", ""]);
const targets = ref<string[]>(["", ""]);

let hoveredElement: BoundedElement | null = null;

onMounted(() => {
  window.addEventListener("resize", handleResize)
})

onUnmounted(() => {
  window.removeEventListener("resize", handleResize)
})

function handleResize() {
  console.log("resize")
  connections.forEach((connection) => {
    const sourceRect = connection.source.element?.getBoundingClientRect();
    if (sourceRect) {
      connection.source.connectionPoint = {
        x: sourceRect.right,
        y: sourceRect.top + sourceRect.height / 2,
      };
    }

    const targetRect = connection.target.element?.getBoundingClientRect();
    if (targetRect) {
      connection.target.connectionPoint = {
        x: targetRect.left,
        y: targetRect.top + targetRect.height / 2,
      };
    }
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
    }
    window.removeEventListener("mousemove", updateLine);
    window.removeEventListener("mouseup", stopDrawing);
  }

  window.addEventListener("mousemove", updateLine);
  window.addEventListener("mouseup", stopDrawing);
}

invoke('get_bd_path').then((res) => {
  console.log(res)
})
</script>

<template>
  <v-container fluid class="fill-height">
    <v-row class="d-flex justify-space-between">
      <v-col cols="4">
        <div>
          <v-img v-for="source in sources" @dragstart.prevent="startDrawing"
                 src="https://picsum.photos/1920/1080"></v-img>
        </div>
      </v-col>

      <v-col cols="4">
        <div>
          <v-img v-for="target in targets" @dragstart.prevent @mouseover="mouseOver" @mouseout="mouseOut"
                 src="https://picsum.photos/1920/1080"></v-img>
        </div>
      </v-col>
    </v-row>
    <svg id="svg" class="position-absolute fill-height w-100">
      <line v-for="line in connections" :x1="line.source.connectionPoint.x" :y1="line.source.connectionPoint.y"
            :x2="line.target.connectionPoint.x" :y2="line.target.connectionPoint.y"
            stroke-width="1" stroke="white"/>
    </svg>
  </v-container>
</template>

<style scoped lang="scss">
#svg {
  pointer-events: none;
  left: 0;
}
</style>
