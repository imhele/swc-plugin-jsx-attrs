# swc-plugin-jsx-attrs

此 [swc](http://swc.rs) 插件用于为符合条件的 jsx element 注入参数。

> babel 版本 [UNDERCOVERj/babel-plugins](https://github.com/UNDERCOVERj/babel-plugins/tree/c19c8a28bbc533791cdcc5372770e70299c8e326/packages/babel-plugin-jsx-append-key-value)

## 示例

### 基础用法

```json
{
  "@fixture/my-components": {
    "Button": [{ "name": "type", "rule": "prepend", "value": "primary" }]
  }
}
```

```diff
  import { Button } from "@fixture/my-components";

  export function MyPage() {
-   return <Button />;
+   return <Button type={"primary"} />;
  }
```

### 支持属性访问语法

```json
{
  "@fixture/my-components": {
    "Dropdown.Link": [{ "name": "type", "rule": "prepend", "value": "primary" }]
  }
}
```

```diff
  import { Dropdown, Noop } from "@fixture/my-components";
  import * as components from "@fixture/my-components";

  export function MyPage() {
    return (
      <>
        <Noop />
-       <Dropdown.Link />
+       <Dropdown.Link type={"primary"} />
        <components.Noop />
-       <components.Dropdown.Link />
+       <components.Dropdown.Link type={"primary"} />
      </>
    );
  }
```

### 支持基于 swc 内置的 id 引用分析

```json
{
  "@fixture/my-components": {
    "Button": [{ "name": "type", "rule": "prepend", "value": "primary" }],
    "Link": [{ "name": "type", "rule": "prepend", "value": "primary" }]
  }
}
```

```diff
  import { Button } from "@fixture/another-components";
  import { Button as MyButton, Link } from "@fixture/my-components";

  export function MyPage() {
    return (
      <>
-       <Link />
+       <Link type={"primary"} />
-       <MyButton />
+       <MyButton type={"primary"} />
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
```
