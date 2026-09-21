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
  The nominal object model and reified generic classes/interfaces are
  experimental. Generic functions, methods, and traits, interface state, trait
  constants, static properties, property hooks, magic methods, anonymous
  classes, cloning, and serialization remain outside the executable
  contract.
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

Public and protected property names remain unique across an inheritance
hierarchy. A descendant may privately redeclare a parent-private property; the
parent slot remains intact and the child receives a distinct slot. Concrete
classes are flattened with parent slots first, then composed-trait properties,
then class properties.
Inherited slots keep their index. Constant defaults for all flattened
properties run before the effective constructor, and reading a property that
was never initialized is a runtime error. Property defaults may nest at most
128 vector or map levels.

## Interfaces and inheritance

Interfaces are methods-only nominal contracts. An interface may
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
Visibility may stay equal or widen, never narrow. A descendant may redeclare a
parent-private method name as a distinct declaration; code already resolved in
the parent remains bound to the parent method. Constructors follow the same compatibility
rules, are inherited when omitted, and are not invoked implicitly when a child
declares its own constructor.

## Generic classes and interfaces

Classes and interfaces declare invariant type parameters. Each parameter may
have one nominal upper bound, and bounds may refer to parameters from the same
declaration:

```thp
interface Source<T>
{
}

class Box<T, U extends Source<T>>
{
    public T $value;

    public function __construct(T $value)
    {
        $this->value = $value;
    }

    public function value(): T
    {
        return $this->value;
    }
}
```

A generic name always has its exact argument count in a static type position.
Arguments cannot contain `void` and must satisfy substituted bounds. Parameters
may appear in properties, method signatures and bodies, parent classes,
implemented interfaces, and bounds. Member access through a parameter requires
a bound that provides that member.

Generic arguments are invariant: `Source<Dog>` is not assignable to
`Source<Animal>`, even if `Dog` extends `Animal`. Parent and interface arguments
are substituted through the complete hierarchy. Reaching the same generic
interface with different arguments is an error.

Construction accepts explicit arguments (`new Box<int>(1)`) or infers them
from supplied constructor arguments (`new Box(1)`). Inference binds positional,
named, and variadic arguments first, then structurally matches class parameters
inside nominal, `vector`, and `map` types. Repeated occurrences must infer the
same exact type. Defaults, omitted arguments, unions, subtype conversion,
assignment context, and return context do not infer arguments; an
underdetermined or conflicting call must provide explicit arguments.

## Dynamic construction

`new $class(...)` and `new (expression)<T, U>(...)` resolve a class at runtime.
The target expression must have the narrowed static type `string`; it is
evaluated once, followed by each explicit argument once in source order. The
result type is always `mixed`.

```thp
$class: mixed = "App\\Service\\Worker";
if (is_string($class)) {
    $worker = new $class<string>(name: "queue");
    if ($worker instanceof Worker) {
        $worker->run();
    }
}
```

Lookup is exact, case-sensitive, and uses the compiled program's canonical
class names. It does not apply the caller's namespace, normalize a leading
backslash, load source files, autoload, invoke callbacks, or use reflection.
Interfaces, traits, abstract classes, native classes, unknown names, and
inaccessible constructors fail with `Error`; invalid UTF-8 names fail with
`ValueError`. Runtime argument mismatches are type errors.

Explicit generic arguments are checked after lookup. Missing trailing
arguments become `mixed`, excess arguments fail, and every declared bound must
still be satisfied. Constructor defaults, names, variadics, visibility,
inheritance, and property defaults use the same rules as static construction.
Collections used as defaults are freshly materialized for each object. Because
object generic arguments are erased, a parameterized runtime compatibility
check that cannot be proved fails safely.

Named static access to a generic class is explicit, as in
`Box<int>::make(1)`. Inside a generic class, `self`, `parent`, and `static`
retain the lexical instantiation. Runtime objects retain their concrete generic
arguments while sharing one class ID per declaration; no monomorphization occurs, and
`$value instanceof Box` tests that erased ID. `instanceof Box<int>` is rejected.
Generic throwable declarations and generic catch targets are unsupported.
Duplicate parameters are diagnosed at the repeated name. Arity, raw-type,
`void`-argument, and bound failures point to the offending nominal reference;
failed constructor inference points to the complete `new` expression and adds
a note to supply explicit arguments. Parameter defaults, `in`/`out` variance,
intersection bounds, generic functions, generic methods, and generic traits are
not accepted by this milestone.

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
graph. A direct positive `if ($value instanceof Foo)` condition narrows that
local to `Foo` within the branch when `Foo` is non-generic.

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
