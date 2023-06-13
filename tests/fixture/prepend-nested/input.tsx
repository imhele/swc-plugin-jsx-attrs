import { Dropdown, Noop } from "@fixture/my-components";
import * as components from "@fixture/my-components";

export function MyPage() {
  return (
    <>
      <Noop />
      <Dropdown.Link />
      <components.Noop />
      <components.Dropdown.Link />
    </>
  );
}
