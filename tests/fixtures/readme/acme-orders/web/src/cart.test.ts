import { total } from "./cart";

test("adds prices", () => {
  expect(total([200, 300])).toBe(500);
});
