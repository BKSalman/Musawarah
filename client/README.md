# Musawarah Front end

make sure you have [nodejs](https://nodejs.org/en) & [pnpm](https://pnpm.io/) installed (already done if using nix)

working directory: `<root>/client`

## Setup
To install the project dependencies you need to use:
```bash
pnpm install
```

## Developing

Once you've installed dependencies with `pnpm install`, start a development server:

```bash
pnpm run dev

# or start the server and open the app in a new browser tab
pnpm run dev -- --open

# or run both backend and frontend with one command
pnpm run full-dev
```

## Building

To create a production version of your app:

```bash
pnpm run build
```

You can preview the production build with `pnpm run preview`.

> To deploy your app, you may need to install an [adapter](https://kit.svelte.dev/docs/adapters) for your target environment.
