<template>
  <v-app>
    <v-app-bar color="surface" elevation="1" height="64">
      <v-container class="h-100 py-0">
        <v-row align="center" no-gutters class="h-100">
          <v-col cols="auto">
            <div class="logo-bunker-sm me-3">
              <v-img
                :src="logo"
                alt="Urocissa Logo"
                cover
                eager
                width="100%"
                height="100%"
              ></v-img>
            </div>
          </v-col>

          <v-col cols="auto">
            <v-app-bar-title class="font-weight-bold text-h5 text-primary">
              Urocissa
            </v-app-bar-title>
          </v-col>

          <v-spacer></v-spacer>

          <v-col cols="auto" class="d-none d-md-block">
            <v-toolbar-items>
              <v-btn href="#features" color="primary" variant="text"
                >Features</v-btn
              >
              <v-btn href="#download" color="primary" variant="text"
                >Download</v-btn
              >
              <v-btn
                href="https://github.com/hsa00000/urocissa/tree/master/docs"
                target="_blank"
                color="primary"
                variant="text"
              >
                Docs
              </v-btn>
              <v-btn
                href="https://github.com/hsa00000/urocissa"
                target="_blank"
                prepend-icon="mdi-github"
                color="primary"
                variant="text"
              >
                GitHub
              </v-btn>
            </v-toolbar-items>
          </v-col>

          <v-col cols="auto">
            <v-btn icon color="primary" @click="toggleTheme">
              <v-icon :icon="themeIcon"></v-icon>
            </v-btn>
          </v-col>
        </v-row>
      </v-container>
    </v-app-bar>

    <v-main class="bg-background layout-locked">
      <v-sheet color="surface">
        <v-container>
          <v-responsive height="64"></v-responsive>

          <v-row justify="center" align="center">
            <v-col cols="12" md="8">
              <v-row justify="center" no-gutters>
                <v-col cols="auto">
                  <div class="logo-bunker-lg">
                    <v-img
                      :src="logo"
                      alt="Urocissa Logo"
                      cover
                      eager
                      class="logo-glow"
                      width="100%"
                      height="100%"
                    ></v-img>
                  </div>
                </v-col>
              </v-row>

              <v-responsive height="24"></v-responsive>

              <div
                class="text-h2 font-weight-bold text-high-emphasis text-center"
              >
                Urocissa
              </div>

              <v-responsive height="16"></v-responsive>

              <div
                class="text-h5 text-medium-emphasis text-center"
                style="line-height: 1.6"
              >
                A self-hosted gallery designed to serve massive collections.<br />
                Serve <strong>millions of photos</strong> on a 4GB RAM server.
              </div>

              <v-responsive height="32"></v-responsive>

              <v-row justify="center" dense>
                <v-col cols="auto">
                  <v-btn
                    color="primary"
                    size="x-large"
                    href="#download"
                    elevation="2"
                    min-width="160"
                  >
                    Download
                  </v-btn>
                </v-col>
                <v-col cols="auto">
                  <v-btn
                    variant="outlined"
                    size="x-large"
                    href="https://demo.photoserver.tw"
                    target="_blank"
                    prepend-icon="mdi-open-in-new"
                    min-width="160"
                  >
                    Live Demo
                  </v-btn>
                </v-col>
              </v-row>

              <v-responsive height="24"></v-responsive>

              <div class="text-caption text-medium-emphasis text-center">
                Free & Open Source (MIT) • Windows & Linux
              </div>
            </v-col>
          </v-row>

          <v-responsive height="48"></v-responsive>

          <v-row justify="center">
            <v-col cols="12" md="10">
              <v-card elevation="24" rounded="xl" class="overflow-hidden">
                <div class="bunker-16-9">
                  <v-img
                    :src="screenshotSrc"
                    cover
                    alt="Urocissa Demo Screenshot"
                    class="absolute-fill"
                    decoding="async"
                  >
                    <template v-slot:placeholder>
                      <v-row
                        class="fill-height ma-0"
                        align="center"
                        justify="center"
                      >
                        <v-progress-circular
                          indeterminate
                          color="grey-lighten-5"
                        ></v-progress-circular>
                      </v-row>
                    </template>
                  </v-img>
                </div>
              </v-card>
            </v-col>
          </v-row>

          <v-responsive height="64"></v-responsive>
        </v-container>
      </v-sheet>

      <v-container id="features">
        <v-responsive height="64"></v-responsive>

        <v-row justify="center">
          <v-col cols="12" class="text-center">
            <div class="text-h3 font-weight-bold text-high-emphasis">
              Why Urocissa?
            </div>
          </v-col>
        </v-row>

        <v-responsive height="48"></v-responsive>

        <v-row>
          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-flash"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Blazing Fast
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                Built with Rust and Vue. Instantly serve, search, and filter one
                million photos in under a second using an in-memory cached
                database.
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-memory"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Memory Efficient
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                Runs seamlessly on a 4GB RAM server even with 1 million photos.
                Linear memory scaling ensures stability for massive libraries.
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-image-multiple"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Infinite Stream
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                Experience endless scrolling without pagination. Advanced
                virtual scrolling handles millions of items smoothly, overcoming
                browser limits.
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-magnify"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Instant Search
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                Use boolean operators (and, or, not) to search your data
                instantly. Find exactly what you're looking for in milliseconds.
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-rocket-launch"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Easy Deployment
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                No complex dependencies. Simple standalone executable for both
                Windows and Linux. Just download and run.
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" md="4">
            <v-card height="100%" elevation="2">
              <v-card-item>
                <template #prepend>
                  <v-icon
                    color="primary"
                    size="x-large"
                    icon="mdi-lock"
                  ></v-icon>
                </template>
                <v-card-title class="text-h5 font-weight-bold">
                  Self-Hosted
                </v-card-title>
              </v-card-item>
              <v-card-text class="text-body-1">
                You own your data. No external cloud dependencies.
                Privacy-focused and secure.
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <v-responsive height="64"></v-responsive>
      </v-container>

      <v-sheet id="download" color="surface">
        <v-container>
          <v-responsive height="64"></v-responsive>

          <v-row justify="center" class="text-center">
            <v-col cols="12" md="8">
              <v-card-title class="text-h3 font-weight-bold">
                Get Started
              </v-card-title>

              <v-responsive height="24"></v-responsive>

              <div class="text-h6 text-medium-emphasis">
                Download Urocissa for your platform and start organizing your
                collection today.
              </div>

              <v-responsive height="32"></v-responsive>

              <v-row justify="center" align="center" dense>
                <v-col cols="12" md="auto">
                  <v-btn
                    color="blue-darken-2"
                    size="x-large"
                    prepend-icon="mdi-microsoft-windows"
                    :href="windowsDownloadUrl"
                    target="_blank"
                    min-width="280"
                  >
                    Download for Windows (.exe)
                  </v-btn>
                </v-col>
                <v-col cols="12" md="auto">
                  <v-btn
                    color="grey-darken-3"
                    size="x-large"
                    prepend-icon="mdi-linux"
                    :href="linuxDownloadUrl"
                    target="_blank"
                    min-width="280"
                  >
                    Download for Linux (.tar.gz)
                  </v-btn>
                </v-col>
              </v-row>

              <v-responsive height="24"></v-responsive>

              <v-btn
                variant="text"
                size="large"
                prepend-icon="mdi-github"
                href="https://github.com/hsa00000/urocissa"
                target="_blank"
              >
                View Source Code
              </v-btn>
            </v-col>
          </v-row>

          <v-responsive height="64"></v-responsive>
        </v-container>
      </v-sheet>
    </v-main>

    <v-footer class="bg-background">
      <v-container>
        <v-row justify="center" no-gutters>
          <v-col cols="12" class="text-center">
            <v-btn
              variant="text"
              icon="mdi-github"
              href="https://github.com/hsa00000/urocissa"
              target="_blank"
            ></v-btn>
          </v-col>
          <v-col cols="12" class="text-center">
            <div class="text-body-2 text-medium-emphasis">
              &copy; 2024-{{ new Date().getFullYear() }} Urocissa. Licensed
              under MIT.
            </div>
          </v-col>
        </v-row>
      </v-container>
    </v-footer>
  </v-app>
</template>

<script setup lang="ts">
import { computed, shallowRef, onMounted } from "vue";
import { useHead } from "#imports";
import { useTheme } from "vuetify";

import logo from "~/assets/logo.png";
import demoDark from "~/assets/demo_dark.jpg";
import demoLight from "~/assets/demo_light.jpg";

const theme = useTheme();

const windowsDownloadUrl = shallowRef(
  "https://github.com/hsa00000/urocissa/releases",
);
const linuxDownloadUrl = shallowRef(
  "https://github.com/hsa00000/urocissa/releases",
);

onMounted(async () => {
  try {
    const response = await fetch(
      "https://api.github.com/repos/hsa00000/urocissa/releases/latest",
    );
    if (!response.ok) throw new Error("Network response was not ok");

    const data = await response.json();

    // Windows Executable
    const exeAsset = data.assets.find((asset: any) =>
      asset.name.endsWith(".exe"),
    );
    if (exeAsset) windowsDownloadUrl.value = exeAsset.browser_download_url;

    // Linux Binary (.tar.gz)
    const linuxAsset = data.assets.find(
      (asset: any) =>
        asset.name.includes("linux") && asset.name.endsWith(".tar.gz"),
    );
    if (linuxAsset) linuxDownloadUrl.value = linuxAsset.browser_download_url;
  } catch (e) {
    console.error("Failed to fetch latest release", e);
  }
});

const isDark = computed(() => theme.global.current.value.dark);
const themeIcon = computed(() =>
  isDark.value ? "mdi-weather-night" : "mdi-weather-sunny",
);
const screenshotSrc = computed(() => (isDark.value ? demoDark : demoLight));

function toggleTheme() {
  theme.global.name.value = isDark.value ? "light" : "dark";
}

useHead({
  title: "Urocissa - Massive Photo Gallery Server",
  meta: [
    {
      name: "description",
      content:
        "Urocissa is a self-hosted gallery designed to serve massive collections. Serve millions of photos on a 4GB RAM server.",
    },
  ],
  link: [{ rel: "preconnect", href: "https://api.github.com" }],
});
</script>
