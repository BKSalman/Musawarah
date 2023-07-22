import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load = (async ({ fetch, params }) => {
  const verification_id = params.id;
  const res = await fetch(
    `http://localhost:6060/api/v1/users/confirm-email/${verification_id}`,
    {
      credentials: "include",
      method: "POST",
    }
  );
  if (res.status === 200) {
    return { message: "Your email has been verified! You may close this page." }
  } else if (res.status == 410) {
    return { message: "This link has expired, try to resend the email." }
  } else if (res.status == 400) {
    throw redirect(307, "/");
  } else {
    return { message: "Something went wrong." }
  }
}) satisfies PageServerLoad;
