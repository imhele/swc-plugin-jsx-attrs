import { Dropdown, Noop } from "@fixture/my-components";
import * as components from "@fixture/my-components";

export function MyPage() {
  return (
    <>
      <Noop onClick={console.log} />
      <Dropdown.Link onClick={console.log} />
      <components.Noop onClick={console.log} />
      <components.Dropdown.Link onClick={console.log} />
    </>
  );
}
