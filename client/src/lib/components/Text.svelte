<script lang="ts">
  import { readTransitions } from "../helpers/transitions";
  import type {
    DrawParams,
    ScaleParams,
    SlideParams,
    CrossfadeParams,
    FadeParams,
    BlurParams,
    FlyParams,
  } from "svelte/transition";
  export let fontSize: "sm" | "md" | "lg" | "xl" = "md";
  export let transitions: {
    in: {
      func: any;
      options:
        | FadeParams
        | BlurParams
        | FlyParams
        | SlideParams
        | ScaleParams
        | DrawParams
        | CrossfadeParams;
    };
    out: {
      func: any;
      options:
        | FadeParams
        | BlurParams
        | FlyParams
        | SlideParams
        | ScaleParams
        | DrawParams
        | CrossfadeParams;
    };
  } | null = null;
  export let className: string = "";

  const { inFunc, inOptions, outFunc, outOptions } =
    readTransitions(transitions);
</script>

<span
  class={`${fontSize} ${className}`}
  in:inFunc={inOptions}
  out:outFunc={outOptions}
>
  <slot />
</span>

<style>
  .sm {
    font-size: 0.75rem;
  }
  .md {
    font-size: 1rem;
  }
  .lg {
    font-size: 1.25rem;
  }
  .xl {
    font-size: 1.75rem;
  }
  span {
    margin: var(--margin);
  }
</style>
