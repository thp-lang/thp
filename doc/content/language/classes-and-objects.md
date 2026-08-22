---
kind: guide
id: guide.languageClassesAndObjects
title: Classes and objects
summary: Describes THP classes, interfaces, properties, methods, and inheritance.
nav:
  section: language
  order: 100
status: experimental
availability: partial
notice: >-
  The nominal object model described here is experimental. Generic interfaces,
  interface state, trait constants, static properties, property hooks, magic
  methods, anonymous classes, cloning, reflection, and serialization remain
  outside the executable contract.
---

A class groups state and behavior. Instances are created with `new`; methods
access the current instance through `$this`.

```thp
class Counter
{

    public int $value;

    public function __construct(int $initial)
    {
        $this->value = $initial;
    }

    public function increment(): int
    {
        $this->value = $this->value + 1;
        return $this->value;
    }
}
```

## Members and visibility

Properties and method boundaries may declare types. Visibility controls access
lexically:

- `public` members are accessible from every scope;
- `protected` members are accessible from their declaring class and its
  descendants;
- `private` members are accessible only from their declaring class.

Trait members become members of the consuming class before these rules are
applied. Both the type checker and bytecode verifier enforce access.

Property names are unique across an inheritance hierarchy; a descendant cannot
replace even a private parent property. Concrete classes are flattened with
parent slots first, then composed-trait properties, then class properties.
Inherited slots keep their index. Constant defaults for all flattened
properties run before the effective constructor, and reading a property that
was never initialized is a runtime error.

## Interfaces and inheritance

Interfaces are non-generic, methods-only nominal contracts. An interface may
extend zero or one interface. A class may extend zero or one class and implement
multiple comma-separated interfaces. Class and interface ancestry is
transitive.

```thp
interface Renderer
{

    public function render(string $value): string;
    public static function kind(): string;
}

interface HtmlRenderer extends Renderer
{
}

interface Cacheable
{

    public function clear(): void;
}

class PageRenderer implements HtmlRenderer, Cacheable
{

    public function render(string $value): string
    {
        return "<p>" . $value . "</p>";
    }

    public static function kind(): string
    {
        return "html";
    }

    public function clear(): void
    {
    }
}

$renderer: Renderer = new PageRenderer();
echo $renderer->render("Hello");
echo PageRenderer::kind();
```

Interface methods end in `;`, have no body, are implicitly abstract, and are
public. Instance and static interface methods are both accepted. Multiple
interfaces may contribute the same requirement only when the complete
signatures are identical.

Multiple direct interface parents are rejected:

```thp
interface Invalid extends First, Second
{
}
```

`abstract class` and abstract methods may retain unresolved inherited, trait,
or interface requirements. Abstract methods end in `;` and cannot be private or
final. A concrete class must resolve every requirement and an abstract class
cannot be instantiated. A final class cannot be extended, and a final method
cannot be replaced by a descendant or a trait.

Overrides are deliberately strict. Staticness, parameter names and order,
parameter types, defaults, variadic shape, and return type must match exactly.
Visibility may stay equal or widen, never narrow. A descendant cannot redeclare
a parent-private method name. Constructors follow the same compatibility
rules, are inherited when omitted, and are not invoked implicitly when a child
declares its own constructor.

## Dispatch and class scope

`$object->method()` and `$this->method()` dispatch from the receiver's runtime
class. Interface-typed calls use the same virtual slots as class-typed calls.
Private and final calls and `self::method()` are lexically bound.
`parent::method()` selects the lexical parent's implementation and supplies
`$this` to an instance method.

A named call such as `Child::method()` starts the late-static context at
`Child`. Forwarding `self::` and `parent::` calls retain that context, while
`static::method()` dispatches from it. In an instance method, `static::` may
select an instance or static method. A static method cannot call an instance
method without an object. Constructors are selected from the effective class
hierarchy and invoked directly.

`instanceof` accepts a class or interface name and follows the complete nominal
graph. It does not currently narrow the static type of the tested expression.

## Traits

Traits are compile-time composition units. They may contain constant-default
properties and concrete or abstract instance and static methods, and may use
other traits. They cannot be instantiated, used as types, implemented, or
extended as classes.

```thp
trait First
{

    public function render(): string
    {
        return "first";
    }
}

trait Second
{

    public function render(): string
    {
        return "second";
    }
}

class Page
{

    use First, Second {
        Second::render insteadof First;
        First::render as protected final legacyRender;
    }
}
```

A class declaration wins over an imported trait method. A selected trait method
wins over an inherited parent method. Two traits contributing the same method
require `insteadof`, even when their signatures match. `as` keeps the original
import and may add an alias or change imported visibility or finality.

Trait properties merge within the current composition only when name, type,
visibility, and constant default are identical. Any other trait-property
conflict, and every collision with an inherited property, is rejected. Trait
bodies are specialized to each consumer, so `$this`, visibility, `parent::`,
and `static::` use the consuming class as their lexical scope.

## Predefined contracts

Language-provided object capabilities are listed under
Predefined interfaces and classes.

## See also

- [Types](thp:guide.languageTypes)
- [Functions](thp:guide.languageFunctions)
- [Namespaces](thp:guide.languageNamespaces)
- [Attributes](thp:guide.languageAttributes)
