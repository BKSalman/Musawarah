import type { BlurParams, CrossfadeParams, DrawParams, FadeParams, FlyParams, ScaleParams, SlideParams } from "svelte/types/runtime/transition";

export function readTransitions(transitions: {
  in: {
    func: any,
    options:
    CrossfadeParams |
    DrawParams |
    ScaleParams |
    SlideParams |
    FlyParams |
    BlurParams |
    FadeParams,
  }
  out: {
    func: any,
    options:
    CrossfadeParams |
    DrawParams |
    ScaleParams |
    SlideParams |
    FlyParams |
    BlurParams |
    FadeParams,
  }
} | null) {
  if (transitions == null) {
    return {
      inFunc: () => { },
      inOptions: null,
      outFunc: () => { },
      outOptions: null,
    };
  }

  return {
    inFunc: transitions.in ? transitions.in.func : () => { },
    inOptions: transitions.in ? transitions.in.options : null,
    outFunc: transitions.out ? transitions.out.func : () => { },
    outOptions: transitions.out ? transitions.out.options : null,
  };
}
