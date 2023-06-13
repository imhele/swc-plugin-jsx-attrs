import { Button } from "@fixture/another-components";
import { Button as MyButton, Link } from "@fixture/my-components";

export function MyPage() {
  return (
    <>
      <Link />
      <MyButton />
    </>
  );
}

export function createPage() {
  function Link() {
    return null;
  }

  return () => (
    <>
      <Link />
      <Button />
    </>
  );
}
