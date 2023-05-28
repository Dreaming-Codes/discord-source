<script lang="ts" setup>
import {appWindow} from "@tauri-apps/api/window";
import {nextTick, onMounted, onUnmounted, reactive, ref, Ref, watch} from "vue";
import {watchArray} from "@vueuse/core";
import {invoke} from "@tauri-apps/api/tauri";
import type {VImg} from "vuetify/components/VImg";
import {SharedUtils} from "../../shared/SharedUtils";

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

let isMounted = false;

const connections = reactive<Connection[]>([]);

interface Target {
}

interface Stream {
  streamPreview: string;
  nickname: string;
}

const sourceElements: Ref<HTMLDivElement[] | null> = ref(null);
const targetElements: Ref<HTMLDivElement[] | null> = ref(null);

const sources = reactive<Map<string, Stream>>(new Map<string, Stream>());
const targets = reactive<Map<string, Target>>(new Map<string, Target>());

//Init with backend streams
invoke("get_streams").then((remote_sources) => {
  Object.entries(remote_sources as Record<string, {
    nickname: string,
    streamPreview: string
  }>).forEach(([streamId, {nickname, streamPreview}]) => {
    sources.set(streamId, {
      streamPreview,
      nickname,
    });
  })
})

//Init with backend targets
invoke("get_targets").then((remote_targets) => {
  console.log(remote_targets);
  Object.entries(remote_targets as { [key: string]: number | null }).forEach(([key, linked_stream]) => {
    if (linked_stream) {

      const unwatch = watch(targetElements, (newTargetElements) => {

        //Search element with matching data-id
        newTargetElements?.every((elem) => {
          if (elem.getAttribute("data-id") == key) {
            //Stop watching for new elements
            unwatch();

            //Get the source element
            const sourceElement = sourceElements?.value?.find((elem) => elem.getAttribute("data-id") == `${linked_stream}`)!;

            connections.push({
              source: {
                element: sourceElement,
                connectionPoint: {
                  x: 0,
                  y: 0,
                }
              },
              target: {
                element: elem,
                connectionPoint: {
                  x: 0,
                  y: 0,
                }
              }
            })

            handleRedraw();

            //Break out of loop
            return false;
          }
        })

      }, {
        flush: "post",
      })
    }


    targets.set(key, {});
  })
})

watchArray([sources, targets], handleRedraw, {
  flush: "post",
})

appWindow.listen("user-info-update", (event) => {
  let payload = event.payload as {
    streamId: string,
    userId: string,
    info: { nickname: string, streamPreview: string }
  }[];
  payload.forEach((update) => {
    const stream = sources.get(update.streamId);
    if (!stream) {
      sources.set(update.streamId, {
        nickname: update.info.nickname,
        streamPreview: update.info.streamPreview,
      })
      return;
    }

    stream.nickname = update.info.nickname;
    stream.streamPreview = update.info.streamPreview;
  });
})

appWindow.listen("stream-removed", (event) => {
  let payload = event.payload as { streamId: string }[];
  payload.forEach((update) => {
    sources.delete(update.streamId);
  });
})

appWindow.listen("web-added", (event) => {
  targets.set(event.payload as string, {});
})

appWindow.listen("web-removed", (event) => {
  targets.delete(event.payload as string);
})

appWindow.listen("discord-disconnected", () => {
  sources.clear();
})

let hoveredElement: BoundedElement | null = null;

onMounted(() => {
  isMounted = true;
  window.addEventListener("resize", handleRedraw)
})

onUnmounted(() => {
  isMounted = false;
  window.removeEventListener("resize", handleRedraw)
})

async function handleRedraw() {
  console.log("Redrawing svg");
  await nextTick();
  connections.forEach((connection, index) => {
    if (connection.target.element && (!document.body.contains(connection.source.element!) || !document.body.contains(connection.target.element!))) {
      connections.splice(index, 1);
      return;
    }

    const sourceRect = connection.source.element?.getBoundingClientRect()!;
    connection.source.connectionPoint = {
      x: sourceRect.right,
      y: sourceRect.top + sourceRect.height / 2,
    };

    //If the line is currently being drawn, use the mouse position as the target
    if (connection.target.element) {
      const targetRect = connection.target.element?.getBoundingClientRect()!;
      connection.target.connectionPoint = {
        x: targetRect.left,
        y: window.scrollY + targetRect.top + targetRect.height / 2,
      };
    }
  })
}

function mouseOver(e: MouseEvent) {
  let element = (e.target as HTMLElement).parentElement!.parentElement!;
  if (!element.dataset.id) {
    element = element.parentElement!;
  }
  const rect = element.getBoundingClientRect();
  hoveredElement = {
    element,
    connectionPoint: {
      x: rect.left,
      y: window.scrollY + rect.top + rect.height / 2,
    }
  };
}

function mouseOut() {
  hoveredElement = null;
}

function startDrawing(e: DragEvent) {
  const targetElement = (e.target as HTMLImageElement);
  const imgRect = targetElement.getBoundingClientRect();

  let currentLine = reactive<Connection>({
    source: {
      element: targetElement,
      connectionPoint: {
        x: imgRect.right,
        y: window.scrollY + imgRect.top + imgRect.height / 2,
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

  async function stopDrawing() {
    if (!hoveredElement) {
      connections.pop();
    } else {
      //Find existing connection on target
      const existingConnectionTarget = connections.filter((connection) => connection.target.element === hoveredElement?.element);
      //Find existing connection on source
      const existingConnectionSource = connections.filter((connection) => connection.source.element === targetElement);

      console.log(existingConnectionSource.length, existingConnectionTarget.length);

      const targetId = hoveredElement?.element?.dataset.id as string;
      const sourceId = targetElement.dataset.id as string;

      //Handle unlink
      if (existingConnectionSource.length > 1 && existingConnectionSource[0].target.element?.dataset.id === targetId &&
          existingConnectionTarget.length > 1 && existingConnectionTarget[0].source.element?.dataset.id === sourceId) {
        appWindow.emit("unlink-stream", {
          target: targetId,
          source: sourceId,
        }).then();
        connections.splice(connections.indexOf(existingConnectionSource[0]), 1);
        connections.pop();
      } else {
        // Unlink any existing connections
        if (existingConnectionSource.length > 1 || existingConnectionTarget.length > 1) {

          if (existingConnectionSource.length > 1) {
            connections.splice(connections.indexOf(existingConnectionSource[0]), 1);

            await appWindow.emit("unlink-stream", {
              target: existingConnectionSource[0].target.element?.dataset.id as string,
            });
          }
          if (existingConnectionTarget.length > 1) {
            connections.splice(connections.indexOf(existingConnectionTarget[0]), 1);

            await appWindow.emit("unlink-stream", {
              target: targetId,
            });
          }

          console.log("Unlinking stream", sourceId, "from", targetId);

          //await SharedUtils.delay(1000)
        }

        // Create a new connection
        appWindow.emit("link-stream", {
          target: targetId,
          source: sourceId,
        }).then();
        console.log("Linking stream", sourceId, "to", targetId);
      }
    }

    window.removeEventListener("mousemove", updateLine);
    window.removeEventListener("mouseup", stopDrawing);
  }

  window.addEventListener("mousemove", updateLine);
  window.addEventListener("mouseup", stopDrawing);
}

function imgLoad() {
  //TODO: Find a way that doesn't require a timeout
  //This is needed for now because vuetify doesn't update the DOM immediately after the image is loaded and using nextTick doesn't work
  setTimeout(handleRedraw, 100);
}

const colors = ["#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#00FFFF", "#FF00FF"];

function getColor(id: number) {
  return colors[id % colors.length];
}
</script>

<template>
  <v-container class="fill-height" fluid>
    <v-row class="d-flex justify-space-between">
      <v-col cols="4">
        <div v-for="[streamId, info] in sources"
             :key="streamId"
             ref="sourceElements"
             :data-id="streamId"
             draggable="true"
             @dragstart.prevent="startDrawing">
          <v-img
              :data-id="streamId"
              :src="info.streamPreview"
              alt=""
              @load="imgLoad">
            <div class="source-target-label">
              {{ info.nickname }}
            </div>
          </v-img>
        </div>
      </v-col>

      <v-col cols="4">
        <div v-for="key in [...targets.keys()].sort()"
             :key="key"
             ref="targetElements"
             :data-id="key"
             draggable="true"
             @mouseout="mouseOut"
             @mouseover="mouseOver"
             @dragstart.prevent>
          <v-img
              :src="'https://picsum.photos/1920/1080?' + key"
              alt=""
              @load="imgLoad">
            <div class="source-target-label">
              {{ key }}
            </div>
          </v-img>
        </div>
      </v-col>
    </v-row>
    <svg id="lineDrawer" class="position-absolute fill-height w-100">
      <line v-for="(line, index) in connections" :stroke="getColor(index)" :x1="line.source.connectionPoint.x"
            :x2="line.target.connectionPoint.x" :y1="line.source.connectionPoint.y"
            :y2="line.target.connectionPoint.y" stroke-width="1"/>
    </svg>
  </v-container>
</template>

<style lang="scss" scoped>
@import "vuetify/styles";

.source-target-label {
  background-color: rgba(0, 0, 0, 0.75);
  @extend .text-white;
  @extend .pa-1;
  @extend .text-body-2;
  @extend .text-center;
}

#lineDrawer {
  pointer-events: none;
  left: 0;
  top: 0;
}
</style>
