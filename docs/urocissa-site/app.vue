<template>
  <v-app>
    <v-app-bar app color="white" elevation="1">
      <v-container class="d-flex align-center py-0">
        <v-avatar rounded="0" class="mr-3">
          <v-img src="/logo.png" alt="Urocissa Logo"></v-img>
        </v-avatar>
        <v-toolbar-title class="font-weight-bold text-h5 text-primary">
          Urocissa
        </v-toolbar-title>

        <v-spacer></v-spacer>

        <div class="d-none d-md-flex">
          <v-btn text href="#features" color="grey-darken-3">Features</v-btn>
          <v-btn text href="#download" color="grey-darken-3">Download</v-btn>
          <v-btn
            text
            href="https://github.com/hsa00000/urocissa/tree/master/docs"
            target="_blank"
            color="grey-darken-3"
            >Docs</v-btn
          >
          <v-btn
            text
            href="https://github.com/hsa00000/urocissa"
            target="_blank"
            prepend-icon="mdi-github"
            color="grey-darken-3"
            >GitHub</v-btn
          >
        </div>
      </v-container>
    </v-app-bar>

    <v-main class="bg-grey-lighten-4">
      <!-- Hero Section -->
      <section class="bg-white pb-16 pt-10">
        <v-container>
          <v-row justify="center" align="center" class="text-center">
            <v-col cols="12" md="8">
              <v-img
                src="/logo.png"
                max-width="150"
                class="mx-auto mb-6"
              ></v-img>
              <h1 class="text-h2 font-weight-bold mb-4 text-grey-darken-4">
                Urocissa
              </h1>
              <p
                class="text-h5 text-medium-emphasis mb-8"
                style="line-height: 1.6"
              >
                A self-hosted gallery designed to serve massive collections.<br />
                Serve <strong>millions of photos</strong> on a 4GB RAM server.
              </p>
              <div class="d-flex justify-center gap-4 flex-wrap">
                <v-btn
                  color="primary"
                  size="x-large"
                  href="#download"
                  elevation="2"
                  class="px-8"
                >
                  Download
                </v-btn>
                <v-btn
                  variant="outlined"
                  size="x-large"
                  href="https://demo.photoserver.tw"
                  target="_blank"
                  class="ml-4 px-8"
                  prepend-icon="mdi-open-in-new"
                >
                  Live Demo
                </v-btn>
              </div>
              <p class="mt-6 text-caption text-medium-emphasis">
                Free & Open Source (MIT) • Windows & Linux (Docker)
              </p>
            </v-col>
          </v-row>
        </v-container>
      </section>

      <!-- Features Section -->
      <section id="features" class="py-16">
        <v-container>
          <h2
            class="text-h3 text-center mb-12 font-weight-bold text-grey-darken-3"
          >
            Why Urocissa?
          </h2>
          <v-row>
            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-flash"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Blazing Fast</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  Built with Rust and Vue. Instantly serve, search, and filter
                  one million photos in under a second using an in-memory cached
                  database.
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-memory"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Memory Efficient</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  Runs seamlessly on a 4GB RAM server even with 1 million
                  photos. Linear memory scaling ensures stability for massive
                  libraries.
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-image-multiple"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Infinite Stream</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  Experience endless scrolling without pagination. Advanced
                  virtual scrolling handles millions of items smoothly,
                  overcoming browser limits.
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-magnify"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Instant Search</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  Use boolean operators (and, or, not) to search your data
                  instantly. Find exactly what you're looking for in
                  milliseconds.
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-docker"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Easy Deployment</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  Simple setup with Docker for Linux or a standalone executable
                  for Windows. Get started in minutes.
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" md="4">
              <v-card height="100%" elevation="2" class="pa-4">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon
                      color="primary"
                      size="x-large"
                      icon="mdi-lock"
                    ></v-icon>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold"
                    >Self-Hosted</v-card-title
                  >
                </v-card-item>
                <v-card-text class="text-body-1">
                  You own your data. No external cloud dependencies.
                  Privacy-focused and secure.
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </v-container>
      </section>

      <!-- Download Section -->
      <section id="download" class="bg-white py-16">
        <v-container>
          <v-row justify="center" class="text-center">
            <v-col cols="12" md="8">
              <h2 class="text-h3 font-weight-bold mb-6">Get Started</h2>
              <p class="text-h6 text-medium-emphasis mb-8">
                Download Urocissa for your platform and start organizing your
                collection today.
              </p>

              <div
                class="d-flex flex-column flex-md-row justify-center gap-4 align-center"
              >
                <v-btn
                  color="blue-darken-2"
                  size="x-large"
                  prepend-icon="mdi-microsoft-windows"
                  href="https://github.com/hsa00000/urocissa/releases"
                  target="_blank"
                  min-width="240"
                  class="ma-2"
                >
                  Download for Windows
                </v-btn>
                <v-btn
                  color="grey-darken-3"
                  size="x-large"
                  prepend-icon="mdi-docker"
                  href="https://github.com/hsa00000/urocissa?tab=readme-ov-file#quick-setup-with-docker"
                  target="_blank"
                  min-width="240"
                  class="ma-2"
                >
                  Setup with Docker (Linux)
                </v-btn>
              </div>
              <div class="mt-6">
                <v-btn
                  variant="text"
                  size="large"
                  prepend-icon="mdi-github"
                  href="https://github.com/hsa00000/urocissa"
                  target="_blank"
                >
                  View Source Code
                </v-btn>
              </div>
            </v-col>
          </v-row>
        </v-container>
      </section>
    </v-main>

    <v-footer class="bg-grey-lighten-3 text-center d-flex flex-column py-8">
      <div>
        <v-btn
          variant="text"
          icon="mdi-github"
          href="https://github.com/hsa00000/urocissa"
          target="_blank"
        ></v-btn>
      </div>
      <div class="pt-2 text-body-2 text-medium-emphasis">
        &copy; {{ new Date().getFullYear() }} Urocissa. Licensed under MIT.
      </div>
    </v-footer>
  </v-app>
</template>

<script setup>
import { useHead } from "#imports";

useHead({
  title: "Urocissa - Massive Photo Gallery Server",
  meta: [
    {
      name: "description",
      content:
        "Urocissa is a self-hosted gallery designed to serve massive collections. Serve millions of photos on a 4GB RAM server.",
    },
  ],
});
</script>

<style>
html {
  scroll-behavior: smooth;
}
</style>
