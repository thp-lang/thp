--TEST--
nominal inheritance, interface dispatch, constructors, visibility, and scoped calls
--FILE--
<?thp

interface Renderer {
    public function render(string $value): string;
    public static function kind(): string;
}

interface HtmlRenderer extends Renderer {}

interface Cacheable {
    public function clear(): void;
}

class BaseRenderer {
    protected string $prefix = "default";

    public function __construct(string $prefix) {
        $this->prefix = $prefix;
    }

    public function label(): string {
        return "base";
    }

    public function lexicalLabel(): string {
        return self::label();
    }

    public function virtualLabel(): string {
        return $this->label();
    }

    public function lateLabel(): string {
        return static::label();
    }

    public function prefixValue(): string {
        return $this->prefix;
    }

    public static function kind(): string {
        return "base";
    }

    public static function forwardedKind(): string {
        return static::kind();
    }

    public static function selfForwardedKind(): string {
        return self::forwardedKind();
    }

    public final function stableKind(): string {
        return static::kind();
    }
}

class PageRenderer extends BaseRenderer implements HtmlRenderer, Cacheable {
    public function render(string $value): string {
        return $this->prefix . ":" . $value;
    }

    public function label(): string {
        return "page";
    }

    public function parentLabel(): string {
        return parent::label();
    }

    public static function kind(): string {
        return "html";
    }

    public static function parentForwardedKind(): string {
        return parent::forwardedKind();
    }

    public function clear(): void {}
}

class ExplicitRenderer extends BaseRenderer {
    public function __construct(string $prefix) {
        parent::__construct($prefix);
    }
}

$renderer: Renderer = new PageRenderer("page");
echo $renderer->render("hello") . "\n";

$page = new PageRenderer("page");
echo $page->lexicalLabel() . "\n";
echo $page->virtualLabel() . "\n";
echo $page->lateLabel() . "\n";
echo $page->parentLabel() . "\n";
echo PageRenderer::forwardedKind() . "\n";
echo PageRenderer::selfForwardedKind() . "\n";
echo PageRenderer::parentForwardedKind() . "\n";
echo $page->stableKind() . "\n";
echo (new ExplicitRenderer("explicit"))->prefixValue() . "\n";
var_dump($page instanceof Renderer);
var_dump($page instanceof HtmlRenderer);
--EXPECT--
page:hello
base
page
page
base
html
html
html
html
explicit
bool(true)
bool(true)
