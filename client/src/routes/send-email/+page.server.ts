import type { PageServerLoad } from './$types';

export const load = (async ({ fetch }) => {
  const res = await fetch(
    "http://localhost:6060/api/v1/users/email_verification",
    {
      credentials: "include",
      method: "POST",
    }
  );
  if (res.status === 200) {
    return { message: "Email Has been sent! Check your inbox or junk folder." }
  } else {
    return { message: "Email failed to be sent" }
  }
}) satisfies PageServerLoad;
