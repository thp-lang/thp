--TEST--
trait properties, nested use, conflict selection, aliases, and consumer specialization
--FILE--
<?thp

trait First {
    protected string $prefix = "trait";

    public function render(): string {
        return "first:" . $this->prefix;
    }
}

trait Second {
    public function render(): string {
        return "second:" . $this->prefix;
    }
}

trait Nested {
    abstract protected function consumerName(): string;

    public function nested(): string {
        return $this->consumerName() . ":" . static::kind();
    }

    public static function traitStatic(): string {
        return static::kind();
    }
}

trait Wrapper {
    use Nested;
}

class ParentPage {
    public function parentValue(): string {
        return "parent";
    }
}

class Page extends ParentPage {
    use First, Second {
        Second::render insteadof First;
        Second::render as public final;
        First::render as protected final legacyRender;
    }
    use Wrapper;

    public static function kind(): string {
        return "page";
    }

    protected function consumerName(): string {
        return "page";
    }

    public function legacy(): string {
        return $this->legacyRender();
    }

    public function fromParent(): string {
        return parent::parentValue();
    }
}

$page = new Page();
echo $page->render() . "\n";
echo $page->legacy() . "\n";
echo $page->nested() . "\n";
echo Page::traitStatic() . "\n";
echo $page->fromParent() . "\n";
--EXPECT--
second:trait
first:trait
page:page
page
parent
